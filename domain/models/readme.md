# Domain Models
These are digital representation of real world things.

Models are specific representations of information.

These are created as statements then refined into one of the categories below.

These are all Objects in an Object Store and Events about them are tracked in an Event Store.

The Models describe as much detail as possible about a single subject.  Each Subject may have many sub-topics.
These are not named at random, their structure is quite important so that we may build a language.

Think of this as an initial dictionary to start understanding how we are breaking things apart by Subject and Limits.

Subjects are the topic or Category involved while Limits are the rules for sizing the Model.

Every model will have a shape, but we don't always determine that in advance. In some cases it is very advantageous to allow the shape to simply emerge from the data.

# Subjects

## Accounts
A place to record transational information.
This involves a combination of organizations, people, policies, entities and values.

## Agents
Automata sacting on behalf of a person

## Arguments
evidence or observations supporting a Claim

## Axioms
Self-evident Laws not imposed by some authority.

some axioms:
  - Things which are equal to the same thing are equal to one another.
  - If equals are added to equals, the wholes are equal.
  - If equals are subtracted from equals, the remainders are equal.
  - Things which coincide with one another are equal to one another.
  - The whole is greater than the part.
  - Things which are double of the same things are equal to one another.
  - Things which are halves of the same things are equal to one another.

## Claims
A statement about a subject. 
There is no truth, falsity or correlation proposed, it is just the claim, such as "The Sky is blue." or "there is always a teapot in my office."

## Environment
Environmental influences, these are things like weather, altitude, power supply and other things we may not have direct influence on, but could have impact on us.

## Equipment
Any physical resource we own or can use as a resource (i.e. a rental).

## Ethics
Moral obligations or commitments we hold as values.

## Facts
Proven claims.
These will have supporting evidence that show a claim is proven from trusted sources.
Some things are not repeatable, such as historic events, or one time events observed, but out of our control. The recording of the observation from a trusted source is usually the basis for the proof.

Facts can be changed, but REQUIRE new obverved evidence supporting the proof of the new claim.

## Funding
Any source of funding that can become liquid capital.
Whether this is money, investment, profit, or donation, it is simply a source of funds we document.

Assets are classified as Equipment, but can create funding through sale or trade.

## Goals
Anything we want to accomplish that is not 

## Ideas
Thoughts, possibly random with no context.
We capture these to see if there are patterns or relationships to make them more relevant.

## Laws
Policies and Procedures imposed by an authority.
Taxes are laws, Gravity is not.

## Locations
Where is something located inrelevance to ->0,0.
Where this is a vector of shperical angle and distance.

We assume ->0,0 is the center of the Earth, but, you are certainly allowed to relocate it.  This will be more relevant over time as we venture further into space and solar system or glactic center becomes more relevant.

## Models
These are anything we can canpture as structured information.

Schemas, Taxonomies, Graphs, Source Code.

### Automata
The word automata comes from the Greek word αὐτόματος, which means "self-acting, self-willed, self-moving". An automaton (automata in plural) is an abstract self-propelled computing device which follows a predetermined sequence of operations automatically.

These are what Agents use as Capabilities.

### Behaviors
A collection of observed or commanded actions resulting in a known outcome.

### Brand
The image you project.
This includes things like Logos, Colors, Slogans, and Mission Statements.

It also includes how you react to Politics, the Ethics that are important to you as an organization, and the Policies you hold important.

### Domain
Domain is a description of the unique capabilities of a named organization or entity.

Color and Location are "domains" and so are "Me" and "Lucy's Lemonade".  The unique characteristics that define them ARE the domain. 

#### Bounded Contexts
simply the combined use of several domains in a transactional context.

#### Subdomains
A domain that can be separated out as a whole, but is a part of a parent domain.

#### Aggregates
Transactional rules involving entities, values, policies and behaviors.

#### Structures
Any form of structured information such as a Category, Schema, or Type.

#### Behaviors
Operations that may be performed.

#### Commands
Instructions for changing the State of a CIM.

#### Entities
Information that has a Unique Identifier.

#### Events
Something that happened. A change in State within a CIM.

#### Maps
iterate a function over the properties of an Entity.

#### Messages
Any transmission within the CIM.

#### Projections
A Structured set of information created from a set of Events.

#### Queries
Observation of a particular State of the CIM at a particular Time.

#### Values
Structures that change as a whole.
Primitives are single field properties, where values are normally collections of properties all changed at the same time, but the identity is not important.

Color is a value...
It has more than one property, but changing any of them, changes the color as a whole.

## Operators
An Oganization or Person that is operating a CIM.

## Organizations
A collection of people, organizations and/or agents with a common goal.  The goal is not always defined, or known.

## People
Collections of Humans.

## Policies
Policies that are followed by a Behavior.

## Politics
Political viewpoints; these are usually attached as relationships to people and organizations.

## Preferences
Things we prefer based on a hierarchy.

## Proofs
Facts and methodologies used to prove a Claim.
These should be verifiable and usually repeatable to qualify as proofs. Instructions to repeat the proof MUST be provided to be a "verified" proof, otherwise it is "falsified".

We also try to tracked failed proofs as to not repeat them.

## Proposals
Any proposed action or state change.

## Refusals
Things we absolutely won't do.

We want these so they never come up in suggestions.

## Relationships
Anything related to anything else that we can verify.

## Secrets
Things we do not want to share for any reason. This includes passwords and private keys.

## Solutions
Any method to solve a known Problem.

## Sources
who claimed what?

## Theories
Proposals that have workflow or known methodologies that we can follow. We may want to add several proofs to theories and evolve them into either facts or laws.