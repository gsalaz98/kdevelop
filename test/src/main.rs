use kdevelop;


fn main() {

}

kdevelop::yaml!(
    apiVersion: apps/v1
    kind: Deployment
    metadata:
      name: ABCDEFGHIJKLMNOP
      labels:
        hello-world/v1: abc123
    spec:
      something: else
);