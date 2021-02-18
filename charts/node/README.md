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
helm upgrade --install node charts/node \
    --set "domain=node.example.com" \
    --set "image.tag=0.1.0" \
    --set "nodes.replicas=2" \
    --set "acme_registration_email=my-email@example.com" \
    --set "nodes.network_name=chi" \
    --set "storage_class=do-block-storage" \
    --set "k8s_v19_used=true"
```
