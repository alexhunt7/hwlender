WIP PXE boot automation tool, for use with [Pixiecore](https://github.com/danderson/netboot/tree/master/pixiecore)

The eventual goal is to enable users to "check out" a machine, which will then be taken offline after a period of time.

## Config files:

### machines.yml

Defines the addresses and pre-boot actions to take for a map of machines.

Example:
```yaml
myhost.mydomain.com:
  hostname: myhost.mydomain.com
  ip: 127.0.0.1
  mac: 08:00:27:22:e2:e6
  pre_boot_actions:
    - ipmi:
        address: 10.10.10.10
        username: asdf
        password: qwer
```

### payloads.yml

Defines the PXE payloads to boot.

Example:
```yaml
v3.0.2:
  kernel: my kernel
  initrd:
    - initrd1
    - initrd2
  cmdline: my command
  message: booting v3.0.2
```
