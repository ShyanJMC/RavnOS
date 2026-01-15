# Huginn

Huginn is the RavnOS' sysinit (systen initializer).

It starts each service in a thread.

## Configuration

The service's configuration is in; /etc/huginn/services. If the file do not exists, huginn will try to create one, but if can not will exit returning 1 (one).

### Syntax

```
[service_name] {
	binary = [path]/[binary]
	arguments = [arguments]
}
```

### Positions and priorities

The services are executing in specified order, so start as PID1 the first service, as PID2 the second, and so on.
Because of that you must pay attention because some services depends of anothers to start properly.

## User and system

Huginn is designed to work in usermode (no root) or system mode (root/sysinit) but always will read the same configuration file; "/etc/huginn/services"
