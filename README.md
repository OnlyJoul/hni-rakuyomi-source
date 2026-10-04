# HNI-Scantrad Aidoku/Rakuyomi source

Eine kleine Source nur für **Hajime no Ippo** auf `https://hni-scantrad.net`.

## Verwendete PizzaReader-Endpunkte

- `GET https://hni-scantrad.net/api/comics/hajime-no-ippo` — Details + Kapitel
- `GET https://hni-scantrad.net/api` + Kapitelpfad — Seitenbilder
- Die allgemeine PizzaReader-API bietet außerdem `/api/comics` und `/api/search/{query}`; diese HNI-only-Source braucht sie nicht.

## Bauen

Benötigt die aktuelle `aidoku-rs` Toolchain / `aidoku` CLI.

```bash
cargo build --release --target wasm32-unknown-unknown -p hniscantrad
```

Je nach installierter Aidoku-CLI kann die `.aix` anschließend aus dem Source-Verzeichnis bzw. dem erzeugten WASM + `res/source.json` gebaut werden. Die aktuelle Aidoku-Dokumentation ist maßgeblich, da die alte pre-0.7 CLI archiviert ist.

## Rakuyomi

Rakuyomi lädt Aidoku-Sources über eine Source-Liste (`index.min.json`). Für den dauerhaften Einsatz daher die gebaute `.aix` zusammen mit einer Source-Liste auf GitHub Pages oder einem anderen HTTP-Server hosten und deren `index.min.json` in Rakuyomi unter `source_lists` eintragen.

## Test

Nach Installation sollte die Suche nach `Hajime no Ippo`, `Ippo` oder `HNI` genau einen Titel liefern. Beim Öffnen werden die Kapitel aus `/api/comics/hajime-no-ippo` geladen; beim Öffnen eines Kapitels werden dessen Seiten über `/api/read/...` geladen.

## Hinweis

Die Source lädt nur die URLs, die HNI-Scantrad selbst über seine PizzaReader-API ausliefert. Wenn der Server seine API-Struktur ändert oder Zugriffe blockiert, muss die Source entsprechend angepasst werden.
