set -e

echo '## OS'
cat /etc/os-release

echo '## Uptime'
uptime

echo '## Disk /'
df -hT /

echo '## Home dirs'
ls -la /home

echo '## azureuser home (top level)'
ls -la /home/azureuser

echo '## Recent files (top 20 by mtime in /home/azureuser)'
find /home/azureuser -maxdepth 3 -type f -printf '%TY-%Tm-%Td %TH:%TM %p\n' 2>/dev/null | sort -r | head -n 20

echo '## Git repos (.git dirs)'
find /home/azureuser -maxdepth 4 -type d -name .git -print 2>/dev/null | sed 's|/\.git$||'

echo '## Running services (top 30)'
systemctl list-units --type=service --state=running --no-pager | head -n 30

echo '## Docker (if installed)'
if command -v docker >/dev/null 2>&1; then docker ps -a --format 'table {{.Names}}\t{{.Image}}\t{{.Status}}'; else echo 'docker not installed'; fi