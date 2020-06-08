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
  ipmi:
    address: 10.10.10.10
    username: asdf
    password: qwer
otherhost.mydomain.com:
  mac: 08:00:27:22:e2:e7
```

In this example, the `hostname`, `ip`, and `ipmi` fields are all optional.

If the `ipmi` field is not provided, the machine may still be PXE booted, but rebooting into PXE mode will have to be done manually.

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

## TODO

- [ ] Tests.
- [ ] Better error messages and handling.
- [ ] Better logging (replace println! statements).
- [X] Make non-essential machine information optional.
- [X] Move all IPMI configs into machine.
- [ ] Support multiple network interfaces.