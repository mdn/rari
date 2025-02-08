To update `html.rs` when upgrading comrak:

```sh
export FROM=from-version
export TO=to-version
curl -o /tmp/html.rs.${FROM} https://github.com/kivikakk/comrak/raw/refs/tags/v${FROM}/src/html.rs
curl -o /tmp/html.rs.${TO} https://github.com/kivikakk/comrak/raw/refs/tags/v${TO}/src/html.rs
git merge-file src/html.rs /tmp/html.rs.${FROM} /tmp/html.rs.${TO}
```
