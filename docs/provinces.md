# 地域一覧

## 凡例

- **種別**: `Coast`（沿岸）/ `Inland`（内陸）/ `Water`（海域）
- **SC**: 補給都市（supply center）
- **初期保有国**: SC の初期保有国（中立 SC は `—`）
- **陸軍隣接**: 陸軍が移動可能な隣接地域コード
- **海軍隣接**: 海軍が移動可能な隣接地域コード（海岸バリアントコードを含む）
- 双海岸地域（`spa`/`stp`/`bul`）は基底コード行に加え、各バリアント行（`_nc`/`_sc`/`_ec`）を別途記載

| コード | 英語名 | 日本語名 | 種別 | SC | 初期保有国 | 陸軍隣接 | 海軍隣接 |
|---|---|---|---|---|---|---|---|
| `adr` | Adriatic Sea | アドリア海 | Water |  |  | — | `alb` `apu` `ion` `tri` `ven` |
| `aeg` | Aegean Sea | エーゲ海 | Water |  |  | — | `bul_sc` `con` `eas` `gre` `ion` `smy` |
| `alb` | Albania | アルバニア | Coast |  |  | `gre` `ser` `tri` | `adr` `gre` `ion` `tri` |
| `ank` | Ankara | アンカラ | Coast | ✓ | Turkey | `arm` `con` `smy` | `arm` `bla` `con` |
| `apu` | Apulia | アプリア | Coast |  | Italy | `nap` `rom` `ven` | `adr` `ion` `nap` `ven` |
| `arm` | Armenia | アルメニア | Coast |  | Turkey | `ank` `sev` `smy` `syr` | `ank` `bla` `sev` |
| `bal` | Baltic Sea | バルト海 | Water |  |  | — | `ber` `bot` `den` `kie` `lvn` `pru` `swe` |
| `bar` | Barents Sea | バレンツ海 | Water |  |  | — | `nrg` `nwy` `stp_nc` |
| `bel` | Belgium | ベルギー | Coast | ✓ | — | `bur` `hol` `pic` `ruh` | `eng` `hol` `nth` `pic` |
| `ber` | Berlin | ベルリン | Coast | ✓ | Germany | `kie` `mun` `pru` `sil` | `bal` `kie` `pru` |
| `bla` | Black Sea | 黒海 | Water |  |  | — | `ank` `arm` `bul_ec` `con` `rum` `sev` |
| `boh` | Bohemia | ボヘミア | Inland |  | Austria | `gal` `mun` `sil` `tyr` `vie` | — |
| `bot` | Gulf of Bothnia | ボスニア湾 | Water |  |  | — | `bal` `fin` `lvn` `stp_sc` `swe` |
| `bre` | Brest | ブレスト | Coast | ✓ | France | `gas` `par` `pic` | `eng` `gas` `mid` `pic` |
| `bud` | Budapest | ブダペスト | Inland | ✓ | Austria | `gal` `rum` `ser` `tri` `vie` | — |
| `bul` | Bulgaria | ブルガリア | Coast | ✓ | — | `con` `gre` `rum` `ser` | — |
| `bul_ec` | Bulgaria(EC) | ブルガリア(EC) | Coast |  |  | — | `bla` `con` `rum` |
| `bul_sc` | Bulgaria(SC) | ブルガリア(SC) | Coast |  |  | — | `aeg` `con` `gre` |
| `bur` | Burgundy | ブルゴーニュ | Inland |  | France | `bel` `gas` `mar` `mun` `par` `pic` `ruh` | — |
| `cly` | Clyde | クライド | Coast |  | England | `edi` `lvp` | `edi` `lvp` `nat` `nrg` |
| `con` | Constantinople | コンスタンティノープル | Coast | ✓ | Turkey | `ank` `bul` `smy` | `aeg` `ank` `bla` `bul_ec` `bul_sc` `smy` |
| `den` | Denmark | デンマーク | Coast | ✓ | — | `kie` `swe` | `bal` `hel` `kie` `nth` `ska` `swe` |
| `eas` | Eastern Mediterranean | 東地中海 | Water |  |  | — | `aeg` `ion` `smy` `syr` |
| `edi` | Edinburgh | エディンバラ | Coast | ✓ | England | `cly` `lvp` `yor` | `cly` `nrg` `nth` `yor` |
| `eng` | English Channel | イギリス海峡 | Water |  |  | — | `bel` `bre` `iri` `lon` `mid` `nth` `pic` `wal` |
| `fin` | Finland | フィンランド | Coast |  | Russia | `nwy` `stp` `swe` | `bot` `stp_sc` `swe` |
| `gal` | Galicia | ガリツィア | Inland |  | Austria | `boh` `bud` `rum` `sil` `ukr` `vie` `war` | — |
| `gas` | Gascony | ガスコーニュ | Coast |  | France | `bre` `bur` `mar` `par` `spa` | `bre` `mid` `spa_nc` |
| `gre` | Greece | ギリシア | Coast | ✓ | — | `alb` `bul` `ser` | `aeg` `alb` `bul_sc` `ion` |
| `hel` | Helgoland Bight | ヘルゴラント湾 | Water |  |  | — | `den` `hol` `kie` `nth` |
| `hol` | Holland | オランダ | Coast | ✓ | — | `bel` `kie` `ruh` | `bel` `hel` `kie` `nth` |
| `ion` | Ionian Sea | イオニア海 | Water |  |  | — | `adr` `aeg` `alb` `apu` `eas` `gre` `nap` `tun` `tyn` |
| `iri` | Irish Sea | アイリッシュ海 | Water |  |  | — | `eng` `lvp` `mid` `nat` `wal` |
| `kie` | Kiel | キール | Coast | ✓ | Germany | `ber` `den` `hol` `mun` `ruh` | `bal` `ber` `den` `hel` `hol` |
| `lon` | London | ロンドン | Coast | ✓ | England | `wal` `yor` | `eng` `nth` `wal` `yor` |
| `lvn` | Livonia | リヴォニア | Coast |  | Russia | `mos` `pru` `stp` `war` | `bal` `bot` `pru` `stp_sc` |
| `lvp` | Liverpool | リヴァプール | Coast | ✓ | England | `cly` `edi` `wal` `yor` | `cly` `iri` `nat` `wal` |
| `gol` | Gulf of Lyon | リオン湾 | Water |  |  | — | `mar` `pie` `spa_sc` `tus` `tyn` `wes` |
| `mar` | Marseilles | マルセイユ | Coast | ✓ | France | `bur` `gas` `pie` `spa` | `gol` `pie` `spa_sc` |
| `mid` | Mid-Atlantic Ocean | 中大西洋 | Water |  |  | — | `bre` `eng` `gas` `iri` `naf` `nat` `por` `spa_nc` `spa_sc` `wes` |
| `mos` | Moscow | モスクワ | Inland | ✓ | Russia | `lvn` `sev` `stp` `ukr` `war` | — |
| `mun` | Munich | ミュンヘン | Inland | ✓ | Germany | `ber` `boh` `bur` `kie` `ruh` `sil` `tyr` | — |
| `naf` | North Africa | 北アフリカ | Coast |  |  | `tun` | `mid` `tun` `wes` |
| `nat` | North Atlantic Ocean | 北大西洋 | Water |  |  | — | `cly` `iri` `lvp` `mid` `nrg` |
| `nap` | Naples | ナポリ | Coast | ✓ | Italy | `apu` `rom` | `apu` `ion` `rom` `tyn` |
| `nth` | North Sea | 北海 | Water |  |  | — | `bel` `den` `edi` `eng` `hel` `hol` `lon` `nrg` `nwy` `ska` `yor` |
| `nrg` | Norwegian Sea | ノルウェー海 | Water |  |  | — | `bar` `cly` `edi` `nat` `nth` `nwy` |
| `nwy` | Norway | ノルウェー | Coast | ✓ | — | `fin` `stp` `swe` | `bar` `nrg` `nth` `ska` `stp_nc` `swe` |
| `par` | Paris | パリ | Inland | ✓ | France | `bre` `bur` `gas` `pic` | — |
| `pic` | Picardy | ピカルディ | Coast |  | France | `bel` `bre` `bur` `par` | `bel` `bre` `eng` |
| `pie` | Piedmont | ピエモンテ | Coast |  | Italy | `mar` `tus` `tyr` `ven` | `gol` `mar` `tus` |
| `por` | Portugal | ポルトガル | Coast | ✓ | — | `spa` | `mid` `spa_nc` `spa_sc` |
| `pru` | Prussia | プロイセン | Coast |  | Germany | `ber` `lvn` `sil` `war` | `bal` `ber` `lvn` |
| `rom` | Rome | ローマ | Coast | ✓ | Italy | `apu` `nap` `tus` `ven` | `nap` `tus` `tyn` |
| `ruh` | Ruhr | ルール | Inland |  | Germany | `bel` `bur` `hol` `kie` `mun` | — |
| `rum` | Rumania | ルーマニア | Coast | ✓ | — | `bud` `bul` `gal` `ser` `sev` `ukr` | `bla` `bul_ec` `sev` |
| `ser` | Serbia | セルビア | Inland | ✓ | — | `alb` `bud` `bul` `gre` `rum` `tri` | — |
| `sev` | Sevastopol | セヴァストポリ | Coast | ✓ | Russia | `arm` `mos` `rum` `ukr` | `arm` `bla` `rum` |
| `sil` | Silesia | シレジア | Inland |  | Germany | `ber` `boh` `gal` `mun` `pru` `war` | — |
| `ska` | Skagerrak | スカゲラク海峡 | Water |  |  | — | `den` `nth` `nwy` `swe` |
| `smy` | Smyrna | スミルナ | Coast | ✓ | Turkey | `ank` `arm` `con` `syr` | `aeg` `con` `eas` `syr` |
| `spa` | Spain | スペイン | Coast | ✓ | — | `gas` `mar` `por` | — |
| `spa_nc` | Spain(NC) | スペイン(NC) | Coast |  |  | — | `gas` `mid` `por` |
| `spa_sc` | Spain(SC) | スペイン(SC) | Coast |  |  | — | `gol` `mar` `mid` `por` `wes` |
| `stp` | St. Petersburg | サンクトペテルブルク | Coast | ✓ | Russia | `fin` `lvn` `mos` `nwy` | — |
| `stp_nc` | St. Petersburg(NC) | サンクトペテルブルク(NC) | Coast |  |  | — | `bar` `nwy` |
| `stp_sc` | St. Petersburg(SC) | サンクトペテルブルク(SC) | Coast |  |  | — | `bot` `fin` `lvn` |
| `swe` | Sweden | スウェーデン | Coast | ✓ | — | `den` `fin` `nwy` | `bal` `bot` `den` `fin` `nwy` `ska` |
| `syr` | Syria | シリア | Coast |  | Turkey | `arm` `smy` | `eas` `smy` |
| `tri` | Trieste | トリエステ | Coast | ✓ | Austria | `alb` `bud` `ser` `tyr` `ven` `vie` | `adr` `alb` `ven` |
| `tun` | Tunis | チュニス | Coast | ✓ | — | `naf` | `ion` `naf` `tyn` `wes` |
| `tus` | Tuscany | トスカーナ | Coast |  | Italy | `pie` `rom` `ven` | `gol` `pie` `rom` `tyn` |
| `tyr` | Tyrolia | ティロル | Inland |  | Austria | `boh` `mun` `pie` `tri` `ven` `vie` | — |
| `tyn` | Tyrrhenian Sea | ティレニア海 | Water |  |  | — | `gol` `ion` `nap` `rom` `tun` `tus` `wes` |
| `ukr` | Ukraine | ウクライナ | Inland |  | Russia | `gal` `mos` `rum` `sev` `war` | — |
| `ven` | Venice | ヴェネツィア | Coast | ✓ | Italy | `apu` `pie` `rom` `tri` `tus` `tyr` | `adr` `apu` `tri` |
| `vie` | Vienna | ウィーン | Inland | ✓ | Austria | `boh` `bud` `gal` `tri` `tyr` | — |
| `wal` | Wales | ウェールズ | Coast |  | England | `lon` `lvp` `yor` | `eng` `iri` `lon` `lvp` |
| `war` | Warsaw | ワルシャワ | Inland | ✓ | Russia | `gal` `lvn` `mos` `pru` `sil` `ukr` | — |
| `wes` | Western Mediterranean | 西地中海 | Water |  |  | — | `gol` `mid` `naf` `spa_sc` `tun` `tyn` |
| `yor` | Yorkshire | ヨークシャー | Coast |  | England | `edi` `lon` `lvp` `wal` | `edi` `lon` `nth` |
