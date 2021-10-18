
fn main() {
    let dep = kdevelop::yaml!(
        apiVersion: apps/v1
        kind: Deployment
        metadata:
          name: ABCDEFGHIJKLMNOP
          labels:
            hello-world/v1: abc123
        spec:
          replicas: 5
          selector:
            matchLabels:
              app: my-application-123
              another: thing
          template:
            metadata:
              labels:
                app: my-deployment-applicatio
            spec: 
              containers:
              - name: nginx
                image: nginx:1.14.2
                ports:
                - containerPort: 80
    );
}