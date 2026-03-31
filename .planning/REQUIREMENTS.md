# Requirements: Spoon Backend Refactoring

**Defined:** 2026-03-28
**Core Value:** 璁?`spoon-backend` 鎴愪负鍞竴鍙俊鐨勫悗鍙版牳蹇冨眰锛岄噸瑕佸姩浣滈兘鍦ㄥ悗绔畬鎴愶紝`spoon` 鍙礋璐ｅ墠绔紪鎺掍笌鍛堢幇銆?
## v1 Requirements

### Backend Boundary

- [x] **BNDR-01**: `spoon` 涓殑 Scoop 瀹夎銆佹洿鏂般€佸嵏杞戒笌 bucket 鎿嶄綔鍙兘閫氳繃 `spoon-backend` 鏆撮湶鐨勫悗绔帴鍙ｈЕ鍙?- [x] **BNDR-02**: `spoon` 涓殑 Git / bucket 浠撳簱鐩稿叧鎿嶄綔鍙兘閫氳繃 `spoon-backend` 鏆撮湶鐨勫悗绔帴鍙ｈЕ鍙?- [x] **BNDR-03**: `spoon` 涓殑 MSVC 妫€娴嬩笌瀹夎鍔ㄤ綔鍙兘閫氳繃 `spoon-backend` 鏆撮湶鐨勫悗绔帴鍙ｈЕ鍙?- [x] **BNDR-04**: `spoon` 涓嶅啀鐩存帴鎺ㄥ Scoop/MSVC 鍚庡彴杩愯甯冨眬璺緞锛屽彧璐熻矗鎶婇厤缃ソ鐨?`root` 浼犵粰 backend
- [x] **BNDR-05**: `spoon` 娑堣垂 backend 杩斿洖鐨勭粨鏋滄ā鍨嬩笌鏌ヨ妯″瀷锛岃€屼笉鏄噸鏂拌鍙?backend 鐘舵€佹枃浠舵垨閲嶅缓鍚庡彴琛屼负

### Scoop State

- [x] **SCST-01**: `spoon-backend` 涓?Scoop 瀹夎鐘舵€佷繚鐣欎竴濂楀敮涓€銆佽鑼冦€佸彲鎸佷箙鍖栫殑鐘舵€佹ā鍨?- [x] **SCST-02**: 鍖呬俊鎭€佸凡瀹夎鐘舵€併€佸嵏杞借緭鍏ヤ笌 reapply 杈撳叆閮藉彲浠ヤ粠杩欏瑙勮寖鐘舵€佹ā鍨嬪鍑?- [x] **SCST-03**: `spoon-backend/src/scoop/` 涓噸澶嶇殑 Scoop 鐘舵€佹ā鍨嬭鍒犻櫎锛岃€屼笉鏄户缁€氳繃閫傞厤灞傚苟瀛?- [x] **SCST-04**: Scoop 鐘舵€佹寔涔呭寲鍙繚瀛樼湡姝ｅ繀瑕佷笖涓嶅彲鎺ㄥ鐨勪簨瀹烇紝涓嶆妸鍙敱甯冨眬鎺ㄥ鍑虹殑缁濆璺緞纭啓杩涚姸鎬?
### Scoop Lifecycle

- [x] **SCLF-01**: `spoon-backend` 灏?Scoop install 娴佺▼鎷嗘垚娓呮櫚鐨勭敓鍛藉懆鏈熼樁娈碉紝鑰屼笉鏄淮鎸佸崟涓法鍨嬫祦绋嬫枃浠?- [x] **SCLF-02**: `spoon-backend` 灏?Scoop update 娴佺▼绾冲叆鍚屼竴濂楀悗绔敓鍛藉懆鏈熸ā鍨嬶紝鑰屼笉鏄 app 渚цˉ閫昏緫
- [x] **SCLF-03**: `spoon-backend` 灏?Scoop uninstall 娴佺▼绾冲叆鍚屼竴濂楀悗绔敓鍛藉懆鏈熸ā鍨嬶紝鑰屼笉鏄 app 渚цˉ閫昏緫
- [x] **SCLF-04**: command-surface reapply銆乮ntegration reapply銆乸ersist restore/sync銆乭ook 鎵ц閮界敱 backend 鐢熷懡鍛ㄦ湡鍏ュ彛缁熶竴鍗忚皟
- [x] **SCLF-05**: hook銆乸ersist銆乻urface銆乸lanner銆乤cquire 绛夐樁娈垫媶鎴愯仛鐒︽ā鍧楋紝鍑忓皯 `runtime/actions.rs` 寮忓法鍨嬫帶鍒舵祦

### Git Ownership

### SQLite Control Plane

- [x] **SQLCP-01**: `spoon-backend` 寮曞叆 SQLite 浣滀负鎺у埗骞抽潰锛屾壙杞?canonical installed state 涓庢仮澶嶅厓鏁版嵁锛岃€屼笉鏄户缁緷璧栧垎鏁ｇ殑 JSON 鎺у埗鏂囦欢
- [x] **SQLCP-02**: 鏂囦欢绯荤粺缁х画浣滀负杩愯鏃舵暟鎹钩闈紱瀹夎鐩綍銆乣current`銆乸ersist銆乧ache銆乻hims銆乻hortcuts銆乥ucket 浠撳簱涓?manifest 鏈綋涓嶈縼鍏ユ暟鎹簱
- [x] **SQLCP-03**: SQLite 璁块棶閫氳繃 `rusqlite` 涓庝粨搴撹嚜鏈夌殑 tokio 杈圭晫灏佽鍦?store / repository 杈圭晫鍐咃紝涓嶆妸椹卞姩缁嗚妭娉勬紡鍒扮敓鍛藉懆鏈熶笟鍔￠€昏緫
- [x] **SQLCP-04**: `spoon-backend` 淇濇寔 sync-core / async-edge 鏋舵瀯杈圭晫锛涙牳蹇冪姸鎬佽鍒欎笌璁″垝閫昏緫涓嶇洿鎺ユ墽琛屾暟鎹簱/鏂囦欢/缃戠粶/杩涚▼ IO
- [x] **SQLCP-05**: 瀵规棫 JSON 鎺у埗骞抽潰鐘舵€佹墽琛岀洿鎺ュ垏鎹紝涓嶄繚鐣欓暱鏈熷吋瀹瑰眰锛涙畫鐣欐棫鐘舵€侀€氳繃鏄惧紡 repair / manual cleanup 澶勭悊

### Git Ownership

- [x] **GIT-01**: `spoon` 涓嶅啀鐩存帴渚濊禆 `gix`
- [x] **GIT-02**: Git / bucket repo 鐨?clone銆乻ync銆乸rogress 浜嬩欢妗ユ帴鐢?`spoon-backend` 鐙崰
- [x] **GIT-03**: backend 鏆撮湶缁?app 鐨?Git 鐩稿叧鎺ュ彛涓嶆硠婕?`gix` 缁嗚妭锛岃€屾槸杩斿洖 backend 绾у埆鐨勭粨鏋滀笌浜嬩欢

### Layout and Context

- [x] **LAY-01**: `spoon-backend` 鎷ユ湁鏍硅矾寰勬淳鐢熷竷灞€鐨勫崟涓€瀹炵幇锛岃鐩?Scoop銆丮SVC 涓庡叡浜?shim/state 甯冨眬
- [x] **LAY-02**: `spoon` 鍙嫢鏈夊簲鐢ㄩ厤缃枃浠惰矾寰勪笌搴旂敤灞傞厤缃涔夛紝涓嶅啀鎷ユ湁鍚庡彴杩愯甯冨眬璇箟
- [x] **LAY-03**: backend 鎿嶄綔鍦ㄦ樉寮忎笂涓嬫枃涓繍琛岋紝涓嶄緷璧栭殣寮忓叏灞€鐜鎴栧垎鏁ｈ矾寰勬帹瀵?
### Testing and Safety

- [x] **TEST-01**: `spoon-backend` 涓?Scoop 鐢熷懡鍛ㄦ湡楂橀闄╄矾寰勮ˉ鍏呭悗绔祴璇曪紝鑷冲皯瑕嗙洊瀹夎銆佹洿鏂般€佸嵏杞戒腑鐨勫叧閿け璐ヨ矾寰?- [x] **TEST-02**: `spoon` 娴嬭瘯淇濇寔鑱氱劍 CLI/TUI 涓庡簲鐢ㄧ紪鎺掞紝涓嶇户缁壙鎷?backend 缁嗚妭姝ｇ‘鎬х殑鍥炲綊瑕嗙洊
- [x] **TEST-03**: 閲嶆瀯杩囩▼涓柊澧炴垨鏇存柊鐨?backend 鎺ュ彛锛岄兘鏈変笌鍏惰亴璐ｇ浉閭荤殑鑱氱劍娴嬭瘯锛岃€屼笉鏄彧闈犵鍒扮娴佺▼鍏滃簳

## v2 Requirements

### Reliability

- **RELY-01**: Scoop install/update 娴佺▼鏀寔鏇存槑纭殑鍥炴粴鎴?journal 璇箟锛岄伩鍏嶅崐鍒囨崲鐘舵€?- **RELY-02**: backend doctor / diagnostics 鑳借В閲婄姸鎬佹崯鍧忋€佽竟鐣岃繚瑙勬垨閲嶆斁澶辫触鍘熷洜

### MSVC

- **MSVC-01**: 鍦?Scoop 涓绘垬鍦虹ǔ瀹氬悗锛屽 `spoon-backend/src/msvc/` 鍋氭洿绯荤粺鐨勫唴閮ㄦ竻鐞?- **MSVC-02**: 鎶藉彇 Scoop 涓?MSVC 鐪熸鍏变韩鐨勫悗绔ā寮忥紝浣嗗彧鍦?Scoop 杈圭晫宸茬ǔ瀹氬悗杩涜

## Out of Scope

| Feature | Reason |
|---------|--------|
| 涓烘棫鐨勪綆璐ㄩ噺鎶借薄淇濈暀鍏煎灞?| 褰撳墠鏄庣‘閲囩敤鍓嶅悜璁捐锛屼紭鍏堝垹闄ゅ潖鎶借薄 |
| 绗竴闃舵涓诲姩瀹屾垚瀹屾暣鐨?MSVC 娣卞害閲嶆瀯 | 褰撳墠涓绘垬鍦烘槸 `spoon-backend/src/scoop/` |
| 鍦?backend 娓呯悊鍓嶅厛鍋氭柊鐨?UI/浜や簰鎵╁睍 | 鐜伴樁娈典环鍊间笉濡傝竟鐣屼笌閲嶅鏀舵暃 |
| 璁?`spoon` 缁х画鐩存帴渚濊禆 `gix` 鎴栭噸寤?Git 琛屼负 | 涓庣洰鏍囪竟鐣岀浉鍐茬獊 |
| 鍚屾椂淇濈暀澶氬 Scoop persisted state 妯″瀷 | 杩欐鏄湰杞浼樺厛娑堢伃鐨勯噸澶?|

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| BNDR-01 | Phase 1 | Complete |
| BNDR-02 | Phase 1 | Complete |
| BNDR-03 | Phase 1 | Complete |
| BNDR-04 | Phase 1 | Complete |
| BNDR-05 | Phase 1 | Complete |
| SCST-01 | Phase 2 | Complete |
| SCST-02 | Phase 2 | Complete |
| SCST-03 | Phase 2 | Complete |
| SCST-04 | Phase 2 | Complete |
| SQLCP-01 | Phase 02.1 | Complete |
| SQLCP-02 | Phase 02.1 | Complete |
| SQLCP-03 | Phase 02.1 | Complete |
| SQLCP-04 | Phase 02.1 | Complete |
| SQLCP-05 | Phase 02.1 | Complete |
| SCLF-01 | Phase 3 | Complete |
| SCLF-02 | Phase 3 | Complete |
| SCLF-03 | Phase 3 | Complete |
| SCLF-04 | Phase 3 | Complete |
| SCLF-05 | Phase 3 | Complete |
| GIT-01 | Phase 1 | Complete |
| GIT-02 | Phase 1 | Complete |
| GIT-03 | Phase 1 | Complete |
| LAY-01 | Phase 1 | Complete |
| LAY-02 | Phase 1 | Complete |
| LAY-03 | Phase 5 | Complete |
| TEST-01 | Phase 4 | Complete |
| TEST-02 | Phase 5 | Complete |
| TEST-03 | Phase 5 | Complete |

**Coverage:**
- v1 requirements: 28 total
- Mapped to phases: 28
- Unmapped: 0

---
*Requirements defined: 2026-03-28*
*Last updated: 2026-03-31 after Phase 4 completion*
