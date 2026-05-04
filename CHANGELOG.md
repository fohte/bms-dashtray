# Changelog

## [0.1.4](https://github.com/fohte/bms-dashtray/compare/v0.1.3...v0.1.4) (2026-05-04)


### Dependencies

* update dependency @tauri-apps/plugin-updater to v2.10.1 ([#102](https://github.com/fohte/bms-dashtray/issues/102)) ([67967a4](https://github.com/fohte/bms-dashtray/commit/67967a476fe50a2c0b8b50ef4e8dd95beb3894b8))
* update dependency recharts to v3.8.1 ([#96](https://github.com/fohte/bms-dashtray/issues/96)) ([0d171c9](https://github.com/fohte/bms-dashtray/commit/0d171c98cc3a05786034ee448e5f84b09e0630e7))
* update rust crate tauri-plugin-updater to v2.10.1 ([#103](https://github.com/fohte/bms-dashtray/issues/103)) ([2ae8f9b](https://github.com/fohte/bms-dashtray/commit/2ae8f9bc53965cdc36ad13bdd034ed374f5f24cc))

## [0.1.3](https://github.com/fohte/bms-dashtray/compare/v0.1.2...v0.1.3) (2026-04-03)


### Features

* **frontend:** add version display and manual update check to settings screen ([#90](https://github.com/fohte/bms-dashtray/issues/90)) ([ad7d4f7](https://github.com/fohte/bms-dashtray/commit/ad7d4f787a8e70c8ec6f4beb464482e94e29b4a0))
* **frontend:** apply `backgroundTransparent` setting to window background ([#86](https://github.com/fohte/bms-dashtray/issues/86)) ([94b9671](https://github.com/fohte/bms-dashtray/commit/94b96716def789f00442d132dc17d9272fc7ba17))
* **play-history:** distinguish mid-play retirement from completed Failed for clear=1 ([#82](https://github.com/fohte/bms-dashtray/issues/82)) ([32925c1](https://github.com/fohte/bms-dashtray/commit/32925c1c98fd526f3cd5ee701cec242b3f3c65ed))


### Bug Fixes

* **backend:** align .bmt parser with beatoraja's actual cache structure ([#62](https://github.com/fohte/bms-dashtray/issues/62)) ([a54d65f](https://github.com/fohte/bms-dashtray/commit/a54d65ffd58650923a24853f88d2054df48c26e1))
* **backend:** align `tauri-plugin-updater` version with frontend JS plugin ([#68](https://github.com/fohte/bms-dashtray/issues/68)) ([a82ce70](https://github.com/fohte/bms-dashtray/commit/a82ce70319e4494a4f9935ad746b1676e09bdeaf))
* **backend:** enable updater artifact generation in bundle config ([#70](https://github.com/fohte/bms-dashtray/issues/70)) ([62cac01](https://github.com/fohte/bms-dashtray/commit/62cac01f714e444c690caeda0babc6594170f84c))
* **ci:** delegate nightly `latest.json` generation to `tauri-action` ([#77](https://github.com/fohte/bms-dashtray/issues/77)) ([7772f51](https://github.com/fohte/bms-dashtray/commit/7772f5134ac75841a22f1985a4e1208c633e19fa))
* **diff_detector:** treat beatoraja sentinel values as no previous play ([#84](https://github.com/fohte/bms-dashtray/issues/84)) ([dfaaa43](https://github.com/fohte/bms-dashtray/commit/dfaaa4338726114f8a598633a634aa939624b4ed))
* **frontend:** apply flash animation to previous lamp in split lamp bar ([#89](https://github.com/fohte/bms-dashtray/issues/89)) ([edee1e1](https://github.com/fohte/bms-dashtray/commit/edee1e16aa041009cccf1b04cc5c9dc538b1874d))
* **frontend:** apply font size setting to all app text ([#64](https://github.com/fohte/bms-dashtray/issues/64)) ([0a09a9e](https://github.com/fohte/bms-dashtray/commit/0a09a9e4fae1a9044241705568db282365522f8a))
* **frontend:** move clear lamp animation keyframes to static CSS ([#69](https://github.com/fohte/bms-dashtray/issues/69)) ([54c8c8c](https://github.com/fohte/bms-dashtray/commit/54c8c8cefad70b8f67c9028dea47f26b5cf9667d))
* **frontend:** pass filtered records to `DistributionChart` ([#88](https://github.com/fohte/bms-dashtray/issues/88)) ([aa80928](https://github.com/fohte/bms-dashtray/commit/aa80928df72e25e55e809dc261631108fd28cc22))
* **table_reader:** resolve md5-only .bmt entries to sha256 via songdata.db ([#83](https://github.com/fohte/bms-dashtray/issues/83)) ([835b438](https://github.com/fohte/bms-dashtray/commit/835b438dc9bf34fb696b4c105e39321f7ce64d14))


### Performance Improvements

* **pipeline:** limit scoredatalog and score DB reads to today only ([#87](https://github.com/fohte/bms-dashtray/issues/87)) ([31542b4](https://github.com/fohte/bms-dashtray/commit/31542b413734952e9c0d079d15ad20aff9d3594a))

## [0.1.2](https://github.com/fohte/bms-dashtray/compare/v0.1.1...v0.1.2) (2026-03-25)


### Features

* enable Tauri auto-updater ([#50](https://github.com/fohte/bms-dashtray/issues/50)) ([7177981](https://github.com/fohte/bms-dashtray/commit/717798171759c4865af386763d0ce6f065f43181))


### Bug Fixes

* **backend:** interpret `scoredatalog.db` timestamps as UNIX seconds ([#51](https://github.com/fohte/bms-dashtray/issues/51)) ([a343b8c](https://github.com/fohte/bms-dashtray/commit/a343b8cc58d1b4efbb366c649d161a9efc5058ca))

## [0.1.1](https://github.com/fohte/bms-dashtray/compare/v0.1.0...v0.1.1) (2026-03-22)


### Bug Fixes

* **setup:** auto-detect player directory during setup ([#47](https://github.com/fohte/bms-dashtray/issues/47)) ([161bf8b](https://github.com/fohte/bms-dashtray/commit/161bf8b30a9d21404a7c3b26f9e5a8a1b39cb1ac))

## [0.1.0](https://github.com/fohte/bms-dashtray/compare/v0.1.0...v0.1.0) (2026-03-20)


### ⚠ BREAKING CHANGES

* Update Rust crate notify to v8 ([#6](https://github.com/fohte/bms-dashtray/issues/6))

* trigger first release ([2e046c5](https://github.com/fohte/bms-dashtray/commit/2e046c5ceae70eafc1fbf87b7cce39e100a14dc9))


### Features

* display difficulty table levels in play history ([#37](https://github.com/fohte/bms-dashtray/issues/37)) ([84ffb3d](https://github.com/fohte/bms-dashtray/commit/84ffb3dc1203e46921b0ea772f1a98930f87d117))


### Bug Fixes

* **backend:** replace default app icons with designed icons ([#34](https://github.com/fohte/bms-dashtray/issues/34)) ([7ba9b65](https://github.com/fohte/bms-dashtray/commit/7ba9b65749156256daa66435f63a8c63e4be41a6))


### Dependencies

* update rust crate chrono to v0.4.44 ([#18](https://github.com/fohte/bms-dashtray/issues/18)) ([0a7d326](https://github.com/fohte/bms-dashtray/commit/0a7d32679e802b50ea5ccfa32839b771909e8ba7))
* update rust crate flate2 to v1.1.9 ([#39](https://github.com/fohte/bms-dashtray/issues/39)) ([a8dfb4a](https://github.com/fohte/bms-dashtray/commit/a8dfb4adb1c41cd09b379563f4463e2a8694e3c8))
* update rust crate indoc to v2.0.7 ([#12](https://github.com/fohte/bms-dashtray/issues/12)) ([d777a3d](https://github.com/fohte/bms-dashtray/commit/d777a3dfb2a4bb328fb55a1e120c53698ebc9b34))
* Update Rust crate notify to v8 ([#6](https://github.com/fohte/bms-dashtray/issues/6)) ([7f3ae87](https://github.com/fohte/bms-dashtray/commit/7f3ae87acd4b5ca2cffaae11587507619257ba3d))
* update rust crate rusqlite to v0.38.0 ([#3](https://github.com/fohte/bms-dashtray/issues/3)) ([38a2557](https://github.com/fohte/bms-dashtray/commit/38a2557c6067456b481295a3a5013ac570c290c4))
* update rust crate tempfile to v3.27.0 ([#14](https://github.com/fohte/bms-dashtray/issues/14)) ([312369b](https://github.com/fohte/bms-dashtray/commit/312369bd959005afbb4aa7cfa7a50c406fe64059))
