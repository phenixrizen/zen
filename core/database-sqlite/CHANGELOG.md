# Changelog

## [2.2.0](https://github.com/phenixrizen/zen/compare/zen-database-sqlite-v2.1.0...zen-database-sqlite-v2.2.0) (2026-08-27)


### Features

* **database-sqlite:** pure-Rust SQLite handler for the database node ([a820770](https://github.com/phenixrizen/zen/commit/a8207708dd6088ed06f52e685ea3dc724a0b3a05))
* **database-sqlite:** swap turso for SQLite itself ([63b1e71](https://github.com/phenixrizen/zen/commit/63b1e714db8ebaf7f86aabb6bbe62bc975b94451))
* **database-sqlite:** swap turso for SQLite itself ([558c1cd](https://github.com/phenixrizen/zen/commit/558c1cdae516fdad7e5a7afa3edb0a43c7ab9406))
* **engine:** database node, decision params, and a pure-Rust SQLite handler ([26ef64c](https://github.com/phenixrizen/zen/commit/26ef64c9b72315db1f9b0b2138b1264a2d599079))
* **engine:** support OR grouping in database node conditions ([691dd5e](https://github.com/phenixrizen/zen/commit/691dd5e55847a43db19cba2731fb4f634700a45e))
* **ffi:** register the SQLite driver from Go in one call ([344bef7](https://github.com/phenixrizen/zen/commit/344bef721bca049d3a10906eb9be60b184042c73))
* **migration:** raw path converts every production query, and both forms verify ([c94f8cf](https://github.com/phenixrizen/zen/commit/c94f8cf067a87d94f05fa958112712ff28244d86))


### Bug Fixes

* **database-sqlite:** stop turso installing a global allocator ([68bc30a](https://github.com/phenixrizen/zen/commit/68bc30a5b539834c880b97c3b15c87bb684e54cc))


### Performance

* **database-sqlite:** pool connections and take the registry off the hot path ([3a3113d](https://github.com/phenixrizen/zen/commit/3a3113dd5212a38bd800ee0482f7af0eba9805be))
