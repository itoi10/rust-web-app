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

---

## デプロイ

- デプロイ先
 - DigitalOcean https://www.digitalocean.com/
 - DigitalOcean App Platform というPaaSでデプロイする
 - お金がかかるので使わないときは停止する

DigitalOceanのアカウント作成

doctlインストール
```
brew install doctl
```

APIキー登録
```
doctl auth init
```

GitHub連携

アプリ作成
```
doctl apps create --spec spec.yaml
```

アプリにspec.yamlの更新を適用する場合
```
doctl apps update <アプリID> --spec=spec.yaml
```

DBマイグレート
```
DATABASE_URL=<DigitalOceanのDBの接続文字列> sqlx migrate run
```

アプリ削除
```
doctl apps delete <アプリID>
```
