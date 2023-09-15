# RustによるWebアプリ開発

このリポジトリは、RustでのWebアプリ開発を学んでいるものです。

Rustによるバックエンド開発の実践的入門書『Zero To Production In Rust』の一部が公開されている[サイト](https://www.lpalmieri.com/)を参考に、学習を進めています。

WebフレームワークとしてActix Web、データベース操作にはSQLxを使用しています


## 起動方法

PostgreSQLをDockerで起動
```sh
sh ./scripts/init_db.sh
```

起動
```sh
cargo run
```

テスト
```sh
cargo test
```

---

## Dockerビルド方法

sqlx-data.json生成
```
cargo sqlx prepare -- --lib
```


Dockerビルド
```
docker build --tag web_prod --file Dockerfile .
```

Docker起動
```
docker run -p 8000:8000 web_prod
```
