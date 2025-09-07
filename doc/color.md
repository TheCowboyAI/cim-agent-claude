# Color
There are perhaps more ways to represent colors than we know what to do with. Can't we all just get along?

Seriously...
There are at least a dozen perfectly good ways to represent color, why should I even entertain another.

It's not about making a superior representation of color, but rather one that helps us understand how we use color. We are interested in defining a Domain of Color. This is a bit different than trying to define a way to represent a specific color.

Everything in a CIM eventually distills to a collection of Conceptual Spaces. This mathematical object helps humans and AI translate semantic meaning.

In the Domain of Color, we represent things as vectors.
This vector is a direction and distance from 0, black.

It has a bottom of 0 and can go no lower.
It also has a top of 100% light, we usually refer to this as White.

In between there is an unlimited number of points limited to to [visible spectrum](https://en.wikipedia.org/wiki/Visible_spectrum). Color is meaningless if it is not observable.
For the context of Color, it has to be within the visible spectrum.

While this is a linear spectrum with a beginning and end, we simply curve the line into a circle and it become the base of the cone.

How can we represent this as a real shape?

Why does it have to BE a shape?

We need to contain the limits of our understanding of the idea of color.

I need a CONVEX shape that represents every possible color and every possible name I can give to a specified "region" of color.


### Considering color... 
When I say "purple", if you are from my culture, that means an equal combination of red and blue.
In some cultures, there is no word for purple, so how do we translate this?
Even in most cultures that have no word for purple, there is some way they represent it.
How do I describe purple to a blind person, or even a color-blind person.
If I told you here is color #ff00ff would you say that is "purple"? Perhaps not. What about #440044?

Conceptual Spaces gives us a way to determine regions based on topology. This means "purple" is a 3-Dimensional region and not necessarily ONLY a hex color.  What is the region we call "purple"? Everything NOT called something else in the same context.

The region is also fluid and depends on the total shape constraint, as well as neighboring regions. There are many representations of "palette" as well, how can we easily move between these ideas, without writing a ton of code just to work with color.  We need a consistent representation and for us this is a vector inside a known topology.

Off the top of my head, this is going to include:
  - color names
  - color codes
    - rbg
      - Red, Blue and Green triplets (addative color)
    - cmyk
      - Cyan, Magenta, Yellow, Black (subtractive color)
    - hsb
      - Hue, Saturation, Brightness
        - this is what we use internally, but with vectors...
  - conversions between codes
  - color theory
  - code generation for color configurations
  - palette generation
  - UI integration

This is mostly available with tooling we already have available, we just need to steer it in a way that is more usable as a system. In essence, this is simply defining what we already do into a consistent, evolving workflow.

Color then, as defined in a Conceptual Space, looks like this:

Two inverted cones.

The bottom cone, pointing down, has the color wheel (hue and saturation), with 0 or black at it's point
The top cone, pointing up, has the color wheel with 100% or white at it's point.

The range is visible light as that is the only meaning we have for color.  We know the limits and this is what is important.

I cannot representable undesireable states.

### Key points (requirements)
We use percentage for brightness, because we need to vary the scales
  Sometimes it's 255, sometimes it's not.

The same regions can have multiple names, and that changes overall context. Something can be warm, red, rouge, primary, Base00, RGBA(255,0,0,255), and #FF0000 all at the same time.

The same point can be in many regions.

I can tell what other regions I am in stricly from my vector.

Regions are based on context.

![Conceptual Space of Color]()

## Goals:
  - Store colors as vectors 
  - convert these vectors to any conceivable representation of color, and map between them
  - Define color palettes for use system-wide
  - Create Palettes from Images
  - Create Images from Palettes
  - Find Images with matching Palettes
  - Associate relationships of names to vectors
  - Create regions using veronoi tessalations
  - Index the regions for cross-reference
  - Allow systemwide use of Color Theory

### Open Source Tools
There are several tools already available in open-source, here are just a few:
Open Color:
  - open-source color scheme, optimized for UI like font, background, border, etc.
  - [https://yeun.github.io/open-color/]

### Online Tools
  - [Adobe Color](https://color.adobe.com/)
  - [Material Design](https://material.io/resources/color)

### Themeing Standards
  - [gtk themes](https://www.gnome-look.org/browse?cat=135&ord=latest)
  - 
