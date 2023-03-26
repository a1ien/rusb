# Changes

## 0.9.2
* Random corrections around the code [#127]
* examples: list_devices: Add vendor and product name [#128]
* examples: read_devices: Improve usage [#125]
* context: create rusb `Context` from existing `libusb_context` [#135]
* `new` now uses `from_raw` [#135]
* Fix stack use after scope in tests [#138]
* Fix United Kingdom misspelling in languages docs [#137]
* fields.rs: Make request_type function a const fn [#142]
* Increase endpoint descriptor's lifetime [#149]
* Fix timeout documentation [#151]

[#127]: https://github.com/a1ien/rusb/pull/127
[#128]: https://github.com/a1ien/rusb/pull/128
[#125]: https://github.com/a1ien/rusb/pull/125
[#135]: https://github.com/a1ien/rusb/pull/135
[#138]: https://github.com/a1ien/rusb/pull/135
[#137]: https://github.com/a1ien/rusb/pull/137
[#142]: https://github.com/a1ien/rusb/pull/142
[#149]: https://github.com/a1ien/rusb/pull/149
[#151]: https://github.com/a1ien/rusb/pull/151

## 0.9.1
* impl Ord and PartialOrd for Version [#116]

[#116]: https://github.com/a1ien/rusb/pull/116

## 0.9.0
* Re-export libusb1-sys as ffi [#75]
* impl Debug for DeviceHandle [#78]
* Add bind to libusb_get_next_timeout [#95]
* Add DeviceHandle::into_raw() [#97]
* Improve read_string_descriptor [#99]
* Derive Debug for Context [#103]
* Implement Clone for Device [#104]
* Add Context::interrupt_handle_events() [#101]
* context: add open_device_with_fd() [#106]
* Rewrite hotplug registration. Add `HotplugBuilder` [#110]. And rewrite [#72]
* ConfigDescriptor and InterfaceDescriptor extra return just slice [#111]

[#72]: https://github.com/a1ien/rusb/pull/72
[#75]: https://github.com/a1ien/rusb/pull/75
[#78]: https://github.com/a1ien/rusb/pull/78
[#95]: https://github.com/a1ien/rusb/pull/95
[#97]: https://github.com/a1ien/rusb/pull/97
[#99]: https://github.com/a1ien/rusb/pull/99
[#101]: https://github.com/a1ien/rusb/pull/101
[#103]: https://github.com/a1ien/rusb/pull/103
[#104]: https://github.com/a1ien/rusb/pull/104
[#106]: https://github.com/a1ien/rusb/pull/106
[#110]: https://github.com/a1ien/rusb/pull/110
[#111]: https://github.com/a1ien/rusb/pull/111

## 0.8.1
* Add getters for bRefresh and bSynchAddress [#61]
* Implement Display for Version. [#59]
* Add Device/DeviceHandle::context getter methods [#57]

[#61]: https://github.com/a1ien/rusb/pull/61
[#59]: https://github.com/a1ien/rusb/pull/59
[#57]: https://github.com/a1ien/rusb/pull/57
