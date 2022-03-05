# Viska
A SIP server/framework built in Rust.

**This is a wip, many things will change until it takes the final shape**

## Why
* because building a customized modern SIP service on top of kamailio/asterisk/opensips
as a developer is not intuitive.
* because although there are some other servers like [routr](https://github.com/fonoster/routr), but these are not
made with development in mind, rather they have configuration that you can change.
For instance you can't manipulate incoming or outgoing requests/responses.
* because although there is [nksip](https://github.com/NetComposer/nksip)
it requires you to not only understand how [nkserver](https://github.com/NetComposer/nkserver)
works but also Erlang and OTP in general
* because the only real alternative is [pjsip](https://www.pjsip.org/)
and if you are looking to build a SIP service you should probably look into that.

Having worked many years on HTTP frameworks, I really think that we can do
something better in SIP.

## How
It is built in Rust because that's a low lever, thread safe, performant language
that I know well. It's built based on traits+generics, so that anything can be
overriden at your will.

The idea is to create a small framework around the basic SIP layers, and then
create libraries and implementations based on public traits for each SIP
extension or service that is needed (like Registrar, Auth, Push notifications
(RFC 8599) etc)

## Progress
- [x] SIP general purpose library/parser with types
- [ ] SDP general purpose library/parser with type
- [x] Transport layer
  - [x] Udp transport
  - [ ] Tcp transport
  - [ ] WS transport
- [x] Transaction layer
  - [x] Invite transaction + impl
  - [ ] Non Invite transaction + impl
- [x] TU layer trait
  - [x] Registrar
  - [x] Capabilities
  - [x] Authentication
  - [ ] Dialogs
  - [ ] Sessions
    - [ ] Initiate a session
    - [ ] Modify a session
    - [ ] Terminating a session
  - [ ] Proxy behavior
