use proc_macro;
use proc_macro2;
use quote;
use syn;

#[proc_macro]
pub fn yaml(stream: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let stream: proc_macro2::TokenStream = stream.into();
    let tokens: Vec<proc_macro2::TokenTree> = stream.into_iter().collect();

    if tokens.is_empty() {
        return syn::Error::new(proc_macro2::Span::call_site(), "No YAML to parse")
            .to_compile_error()
            .into();
    }

    let mut lines = Vec::default();
    let mut line = String::default();
    let mut prev_token = &tokens[0];
    let initial_indent_whitespace = prev_token.span().start().column;
    
    for token in tokens.iter() {
        let prev_span = prev_token.span();
        let span = token.span();

        let prev_end = prev_span.end();
        let current_start = span.start();

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
                .checked_sub(initial_indent_whitespace)
                .unwrap_or(current_start.column)
        } else {
            let start_spaces = current_start.column
                .checked_sub(prev_end.column)
                .unwrap_or(current_start.column);

            start_spaces
                .checked_sub(initial_indent_whitespace)
                .unwrap_or(start_spaces)
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
    let yaml_data: serde_yaml::Value = match serde_yaml::from_str(&yaml_spec) {
        Ok(parsed) => parsed,
        Err(err) => return syn::Error::new(proc_macro2::Span::call_site(), format!("Invalid YAML: {:#?}", err))
            .to_compile_error()
            .into()
    };

    let api_version = match yaml_data["apiVersion"].as_str() {
        Some(resource_version) => resource_version,
        None => return syn::Error::new(proc_macro2::Span::call_site(), "API version unable to be parsed as str")
            .to_compile_error()
            .into()
    };

    let api_version_components: Vec<syn::Ident> = api_version.replace("-", "_")
        .split("/")
        .map(|v| quote::format_ident!("{}", v))
        .collect();

    let kind = match yaml_data["kind"].as_str() {
        Some(decl_kind) => decl_kind,
        None => return syn::Error::new(proc_macro2::Span::call_site(), "No kind found")
            .to_compile_error()
            .into()
    };

    let kind_ident = syn::Ident::new(kind, proc_macro2::Span::call_site());
    let yaml_lit = syn::LitStr::new(&yaml_spec, proc_macro2::Span::call_site());

    (quote::quote! {{
        let return_type: ::k8s_openapi::api::#(#api_version_components)::*::#kind_ident = ::serde_yaml::from_str(#yaml_lit).expect("Failed to parse YAML properly");
        return_type
    }}).into()
}