cargo build --release
scp -i ~/.ssh/tihomir-keypair.pem ./target/release/vroomgine ec2-user@ec2-3-70-53-249.eu-central-1.compute.amazonaws.com:/opt/apps/rust/vroomgine.new
ssh -i ~/.ssh/tihomir-keypair.pem ec2-user@ec2-3-70-53-249.eu-central-1.compute.amazonaws.com
sudo systemctl stop vroomgine
cp /opt/apps/rust/vroomgine /opt/apps/rust/vroomgine.bak.$(date +%s)
mv /opt/apps/rust/vroomgine.new /opt/apps/rust/vroomgine
chmod +x /opt/apps/rust/vroomgine
sudo systemctl start vroomgine

cd  /opt/apps/nestjs/
sudo systemctl stop nestjs
npm ci
npm run build
sudo systemctl start nestjs
