#####
Explany backend API

###
1. configure .env
2. run migration `cargo sqlx migrate run`
3. (optional) prepare offline data for sqlx `cargo sqlx prepare`
4. run server `cargo run`
5. test `cargo test`

###
swagger spec 

http://localhost:8081/swagger-ui/index.html?url=/swagger-spec