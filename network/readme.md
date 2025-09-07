# Networking
This is ephemeral using NetworkManager.

# Declarative Network
Set a network segment for the CIM.

This is a Private IP4 Address range to use for the CIM.

Use a big enough space, but more addresses means longer lookup times and suffered network performance.

If you don't plan on more than 255 addresses you only need a single Class C address: 192.168.0.0/24.
You can always upgrade and harden the network later.
Nothing is wide-open, so we are safe to use a single network for most things.

However, if you plan to deploy thousands of machines and containers, feel free to use: 10.0.0.0/8 and implement hardened security immediately.

We will give you graphs to work with... Networking can be difficult, we help make it easier to work with inside a CIM.

192.168.0.0/24 looks weird and scary to many people... Let's understand this a little more.

This is just a number range, albeit presented in a different way.

It seems easier to write than: 3232235520 to 3232235775, but that is all it is, a range of 256 numbers.
All these numerical gymnastics are attempts to make talking about that range easier for engineers.

When I declare the network, I am stating we know what we want these ranges to mean.