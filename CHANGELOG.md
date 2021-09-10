# Changes

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
