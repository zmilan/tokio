# Redis Tutorial

## Introduction

In this tutorial we will go through the basics of writing a redis like database
with `tokio`. We will implement some of the basic redis commands such as `set` and
`get` but we will also extend it to support `subscribe` and `publish`. The end goal
is to have a server that we can interact with via the `redis-cli`.

## Basic layout of KV

### TCP Accept loop

### Database struct / Modeling shared data

### Reading/Writing from a TCP connection

### `AsyncReadBufExt`

#### Handling the Redis protocol

## Pub/Sub

TBW
