# Count your counters

A small game to figure out a good way to implement card countering machanics.

All cards come in one (or more) of `n` flavours.

On a player's turn they use a card to attempt to win. The attempt has a flavour. The cards each allow countering cards of a particular set of flavours. E.g. a card may counter cherry or blue raspberry cards, but be lemon flavoured. Counters can counter other counters.

If the attempt fails, each player draws back up to a full hand, and the next player goes.

Cards are dealt from the same deck with enough cards that at least one full hand left once all the players have a full hand.

For a longer game, the attempts can be attempts to gain points of the given flavour, and you need `k` differently flavoured points to win.