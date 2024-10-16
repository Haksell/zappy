example:

```
cargo run -p server -- -p 8080 -x 20 -y 10 -n anton axel -c 1 -t 4
```

other terminal:

```
nc localhost 8080
BIENVENUE
axel
0
20 10
"Voir"
OK
```

Todo client:

- serialize Command enum using serde_json from workspace cargo.toml
- when read responses serialize ServerResponse enum from workspace cargo.toml 