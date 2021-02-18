# Install dependencies

Required helm v3.x, see https://helm.sh/docs/intro/install.

Required ingress-nginx, see https://kubernetes.github.io/ingress-nginx/deploy.

Required cert-manager:
```bash
kubectl create namespace cert-manager

helm repo add jetstack https://charts.jetstack.io

helm install \
  cert-manager jetstack/cert-manager \
  --namespace cert-manager \
  --version v1.1.0 \
  --set installCRDs=true
```
(see https://cert-manager.io/docs/installation/kubernetes for details)

# Deploy

```bash
helm upgrade explorer charts/explorer \
    --install \
    --set "domain=explorer.example.com" \
    --set "replicas=1" \
    --set "node_ws_url=wss://testnet.social.network" \
    --set "acme_registration_email=my-email@example.com" \
    --set "k8s_v19_used=true"
```
