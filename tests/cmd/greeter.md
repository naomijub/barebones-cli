# Greeter Plugin

## Upper case greeting

```console
$ barebones-cli greeter -- --uppercase julia
[32m INFO[0m [2mopentelemetry[0m[2m:[0m  [3mname[0m[2m=[0m"MeterProvider.GlobalSet" Global meter provider is set. Meters can now be created using global::meter() or global::meter_with_scope().
greeter: HELLO, JULIA!

```

## Simple greeting

```console
$ barebones-cli greeter julia
[32m INFO[0m [2mopentelemetry[0m[2m:[0m  [3mname[0m[2m=[0m"MeterProvider.GlobalSet" Global meter provider is set. Meters can now be created using global::meter() or global::meter_with_scope().
greeter: Hello, julia!

```

## Empty greeting

```console
$ barebones-cli greeter
[32m INFO[0m [2mopentelemetry[0m[2m:[0m  [3mname[0m[2m=[0m"MeterProvider.GlobalSet" Global meter provider is set. Meters can now be created using global::meter() or global::meter_with_scope().
greeter: Hello, World!

```