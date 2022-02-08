## About
SquireCore employs a modified swiss-style pairings algorithm that is generalized to pair `N` players, instead of the traditional two.
This docuement goes into two levels of detail about this modified algorithm.
For even deeper detail, check the source code.

## Goals
The traditional, two-player swiss pairings system seeks to pair players with similar play records against each other.

## Basics
To understand how SquireCore's swiss pairings work, it will first help to understand how traditional swiss pairings work.
In traditional swiss pairings, we start with a list of all active players that is sorted in descending order according to their score (highest scoring players first).
The process for creating the pairings is as follows:
<ol>
 <li>Take the highest placed player from the list</li>
 <li>Walk through the list from start towards the end until you find the first player that the first player has yet to play against</li>
 <li>Remove this player from the list and pair the two removed players against each other</li>
 <li>Repeat this until everyone is paired (or one player is left, who gets a bye)</li>
</ol>

SquireCore's modified swiss pairings works to generalize this, but for pairings of more than two players at a time.
The key difference is in step 2.
If, for example, pairings of four players are being made, SquireCore doesn't stop "walking the list" until it find four player that have never played each other (that is, there are no old opponents).

## In-Depth
The algorithm as described is a "greedy" algorithm, that is it takes the first possible option it sees; however, this can lead into a couple issues, which SquireCore tries to remedy.
The simplest one is that we "walk off the list" (run out of players when pairing).
When this happens, SquireCore adjusts its pairings slightly.

Let's say we are trying to pair player A.
We start walking the list and the first player that we encounter that player A has yet to play against is player B.
We add player B into our potential pairing and continue walking until we find a player that neither player A nor player B have played.
We add this player C to the potential, but it is possible that there isn't a player D.
This can happen when players A, B, and C have largely played against different groups of people.
In this case, SquireCore first removes player C from the potential pairings and starts walking from their position in the list to find a new player C.
If this happens too many time, SquireCore instead removes both players B and C from the potential pairing and continues.
