# Greeter Plugin

## Upper case greeting

```console
$ barebones-cli greeter --verbose -- --uppercase julia
[DEBUG]: barebones -  * Settings: 
name = "Julia Naomi"
is_machine = false
version = "1.0.0"
plugins = ["greeter"]

[DEBUG]: barebones - Loading plugin from: /Users/jnaomi/.barebones/libgreeter.dylib
[INFO]: greeter -   Loaded: greeter v0.1.0
[DEBUG]: greeter -   Description: A simple greeter plugin
[DEBUG]: greeter -   Author: Julia Naomi
greeter: HELLO, JULIA!

```

## Simple greeting

```console
$ barebones-cli greeter julia
[INFO]: greeter -   Loaded: greeter v0.1.0
greeter: Hello, julia!

```

## Empty greeting

```console
$ barebones-cli greeter
[INFO]: greeter -   Loaded: greeter v0.1.0
greeter: Hello, World!

```