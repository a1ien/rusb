# Changes

## 0.6.5
* Support pkg_config for MSVC. [#191]
* Fix package detection and build when cross-compiling from MSVC to GNU [#180]
* libusb_set_iso_packet_lengths panics on debug builds in newest nightly (2024-03-27) [#199]
* Added libusb_free_pollfds() in the available FFI methods. [#203]

[#191]: https://github.com/a1ien/rusb/pull/191
[#180]: https://github.com/a1ien/rusb/pull/180
[#199]: https://github.com/a1ien/rusb/pull/199
[#203]: https://github.com/a1ien/rusb/pull/203

## 0.6.3-0.6.4
* Patch for macOS Big Sur and newer allowing to link statically [#133]
* Add libudev include paths as specified by pkg-config [#140]

[#133]: https://github.com/a1ien/rusb/pull/133
[#140]: https://github.com/a1ien/rusb/pull/140


## 0.6.2
* Rename compiled library when vendored libusb is used [#130]

[#130]: https://github.com/a1ien/rusb/pull/130

## 0.6.1
* Add LIBUSB_OPTION_NO_DEVICE_DISCOVERY constant
* Bump vendored libusb version from 1.0.24 to 1.0.25 [#119]

[#119]: https://github.com/a1ien/rusb/pull/119

## 0.6.0
* Allow null function pointers for libusb_set_log_cb() [#74]
* Allow null function pointers for libusb_set_pollfd_notifiers() [#71]
* Fix building of recent libusb on macOS [#108]
* Ignore vendored feature on FreeBSD [#109]
* Update definitions [#112]

[#74]: https://github.com/a1ien/rusb/pull/74
[#71]: https://github.com/a1ien/rusb/pull/71
[#108]: https://github.com/a1ien/rusb/pull/108
[#109]: https://github.com/a1ien/rusb/pull/109
[#112]: https://github.com/a1ien/rusb/pull/112
