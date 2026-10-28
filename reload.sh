cargo build --release
ls -l target/release/Unity_server
systemctl restart car_unity_api
systemctl status car_unity_api