set -e

echo '## Docker status'
if systemctl is-active --quiet docker; then echo 'docker: active'; else echo 'docker: inactive'; fi

echo '## Docker images'
if command -v docker >/dev/null 2>&1; then docker images --format '{{.Repository}}:{{.Tag}} {{.Size}}'; else echo 'docker not installed'; fi

echo '## Docker containers (all)'
if command -v docker >/dev/null 2>&1; then docker ps -a --format 'table {{.Names}}\t{{.Image}}\t{{.Status}}'; else echo 'docker not installed'; fi

echo '## Kubernetes services (kubelet/k3s/microk8s)'
for svc in kubelet k3s microk8s; do systemctl list-units --type=service --all | grep -q "$svc" && systemctl status "$svc" --no-pager || true; done

echo '## K8s dirs'
ls -la /etc/kubernetes 2>/dev/null || echo '/etc/kubernetes not found'
ls -la /var/lib/kubelet 2>/dev/null || echo '/var/lib/kubelet not found'
ls -la /var/lib/rancher 2>/dev/null || echo '/var/lib/rancher not found'


echo '## Processes (kube/docker/containerd)'
ps aux | egrep -i 'kube|k3s|microk8s|containerd|dockerd' | head -n 50


echo '## Large/interesting files in /home/azureuser (top 20 by size)'
find /home/azureuser -type f -printf '%s %p\n' 2>/dev/null | sort -nr | head -n 20