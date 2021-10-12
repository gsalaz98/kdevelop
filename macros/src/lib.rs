use proc_macro;
use proc_macro2;
use quote;
use syn;
use yaml_rust;

#[proc_macro]
pub fn yaml(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let stream: proc_macro2::TokenStream = stream.into();
    let tokens: Vec<proc_macro2::TokenTree> = stream.into_iter().collect();

    if tokens.is_empty() {
        panic!("No YAML to parse");
    }

    let mut lines = Vec::default();
    let mut line = String::default();
    let mut prev_token = &tokens[0];
    
    for token in tokens.iter() {
        let prev_span = prev_token.span();
        let span = token.span();

        let (prev_start, prev_end) = (prev_span.start(), prev_span.end());
        let (current_start, current_end) = (span.start(), span.end());

        // Check if we're on a new line
        if prev_end.line != current_start.line {
            lines.push(line);
            line = String::default();
        }

        // No whitespace, add string and continue
        if prev_end.column == current_start.column {
            line += &token.to_string();
            prev_token = token;
            continue;
        }

        // Get how much whitespace this line requires, whether it's the 
        // first token of the line or following a previous token on the same line
        let spaces = if prev_end.line != current_start.line {
            current_start.column
        } else {
            current_start.column
                .checked_sub(prev_end.column)
                .unwrap_or(current_start.column)
        };

        line += &(0..spaces)
            .map(|_| " ")
            .collect::<String>();

        line += &token.to_string();
        prev_token = token;
    }

    if !line.is_empty() {
        lines.push(line);
    }

    let yaml_spec = lines.join("\n");

    if false {
        let name_lit = syn::LitStr::new(&yaml_spec, proc_macro2::Span::call_site());
        return (quote::quote! {
            const stuff: &'static str = #name_lit;
        }).into()
    }

    let yaml_data = yaml_rust::YamlLoader::load_from_str(&yaml_spec)
        .expect("Invalid YAML");

    
    let api_version = yaml_data[0]["apiVersion"].as_str().expect("No API version specified");
    let api_version_components: Vec<syn::Ident> = api_version.split("/")
        .map(|v| syn::Ident::new(v, proc_macro2::Span::call_site()))
        .collect();

    let kind = yaml_data[0]["kind"].as_str().expect("No kind found");
    let kind_ident = syn::Ident::new(kind, proc_macro2::Span::call_site());

    (quote::quote! {
        use k8s_openapi::api::#(#api_version_components)::*::#kind_ident;
    }).into()
}