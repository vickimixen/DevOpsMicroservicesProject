# Setting up the API gateway

We are using Kong as our API gateway. https://konghq.com/kong/

## Set up the services etc.
***One-time setup***

Start by changing the "host" param in the `ingress.yaml` and `ingress_block.yaml` files to match whatever hostname you would like to use the for gateway.


## Set up the deployment for the gateway

`kubectl apply -f deployment.yaml`

## Set up the ingress rules for the gateway

`kubectl apply -f ingress.yaml -f block_plugin.yaml -f ingress_block.yaml`

The initial 'ingress.yaml` file looks like this:
```
apiVersion: "extensions/v1beta1"
kind: "Ingress"
metadata:
  name: "legeserver-ingress"
  namespace: "default"
  annotations:
    konghq.com/strip-path: "true"
    kubernetes.io/ingress.class: kong
spec:
  tls:
  rules:
  - host: "api.legeserver.com"
    http:
      paths:
        - path: "/user"
          backend:
            serviceName: "userautograder"
            servicePort: 80
        - path: "/repo"
          backend:
            serviceName: "repository-service"
            servicePort: 80
        - path: "/assignment"
          backend:
            serviceName: "assignment-service"
            servicePort: 80
```

In the rules sections you can add/edit/delete paths and which service they should point to. 
For instance the `/user` path "proxies" all requests to the `userautograder` service.

After editing, you can apply the changes by running `kubectl apply -f ingress.yaml` again and the changes will be available within a few seconds.
