# Unity_server

Unity ゲーム用のスコアランキングサーバーです。Rust (axum) で書かれていて、難易度ごと (easy / hard) にタイムを記録し、上位10件を返します。

## 特徴

- 難易度ごとに別々のランキングを管理 (`easy` / `hard`)
- スコアはタイムの短い順に並べて、上位10件を返す
- スコアは CSV ファイルに保存されるので、再起動しても残る
- 存在しない難易度を指定すると 404 を返す

## 使用技術

- Rust
- [axum](https://github.com/tokio-rs/axum)
- [tokio](https://tokio.rs/)
- [serde](https://serde.rs/)

## 動かし方

```
git clone https://github.com/okonomiyak/Unity_server.git
cd Unity_server
cargo run
```

サーバーは `http://localhost:3030` で起動します。

## API

| メソッド | パス | 説明 |
| --- | --- | --- |
| GET | `/scores/{mode}` | ランキング (上位10件) を取得 |
| POST | `/scores/{mode}` | スコアを追加 |

`{mode}` には `easy` か `hard` を指定します。それ以外は `404 Not Found` になります。

### ランキング取得

```
GET /scores/easy
```

レスポンス例:

```json
{
  "scores": [
    { "name": "test", "time": 12.3 },
    { "name": "taro", "time": 15.8 }
  ]
}
```

`time` が小さい順に並びます。

### スコア追加

```
POST /scores/easy
Content-Type: application/json
```

リクエスト例:

```json
{ "name": "test", "time": 12.3 }
```

追加に成功すると `201 Created` を返します。

## 使い方の例 (PowerShell)

```powershell
# 一覧取得
Invoke-RestMethod http://localhost:3030/scores/easy

# スコア追加
Invoke-RestMethod -Method Post -Uri http://localhost:3030/scores/easy -ContentType "application/json" -Body '{"name":"test","time":12.3}'
```

## 使い方の例 (curl)

```
curl http://localhost:3030/scores/easy
curl -X POST -H "Content-Type: application/json" -d '{"name":"test","time":12.3}' http://localhost:3030/scores/easy
```

## メモ

- スコアを保存する CSV ファイルは `.gitignore` で Git の管理対象から外しています。