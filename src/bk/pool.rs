use core::{cmp, mem};

use arena::Arena;
use bk::search::{self, Search};
use random;

struct Pool {
    head: Node,
    arena: Arena<Node>,
}

impl Pool {
    /// Search the block pool for a particular block.
    ///
    /// The outline of the algorithm is this: We start by shortcutting from the top level until we
    /// overshoot, then we repeat on the next level starting at the last non-overshot shortcut from
    /// the previous level.
    ///
    /// # Example
    ///
    /// If we look for 8, we start in the top level and follow until we hit 9.
    ///     ==================> [6] --- overshoot ----> [9] -----------> NIL
    ///     ------------------> [6] ==> [7] ----------> [9] -----------> NIL
    ///     ----------> [5] --> [6] ==> [7] ----------> [9] --> [10] --> NIL
    ///     --> [1] --> [5] --> [6] --> [7] ==> [8] --> [9] --> [10] --> NIL
    fn search(&mut self, block: &Block) -> Seek {
        log!(DEBUG, "Searching the block pool for block {:?}...", block);

        // Use `BlockSearcher` for this.
        self.search_with(search::BlockSearcher {
            needle: block,
        }).unwrap()
        // TODO: Find a way to fix this unwrap.
    }

    /// Search the block pool with a particular searcher.
    ///
    /// The outline of the algorithm is this: We start by shortcutting from the top level until we
    /// need to refine (which is determined by the serarcher), then we repeat on the next level
    /// starting at the last refined shortcut from the previous level. At the lowest level, we go
    /// forward until we find a match.
    ///
    /// A "lookback", i.e. the refined nodes of every level is stored in the returned value.
    fn search_with<S: Search>(&mut self, searcher: S) -> Result<Seek, ()> {
        // We start by an uninitialized value, which we fill out.
        let mut seek = unsafe { mem::uninitialized() };

        // Start at the highest (least dense) level.
        let mut iter = self.head.follow_shortcut(lv::Level::max());
        // Go forward until we can refine (e.g. we overshoot).
        while let Some(shortcut_taken) = iter.find(|x| searcher.refine(x)) {
            // Decrement the level.
            let lv::Level(lv) = iter.decrement_level();
            log!(INTERNAL, "Going from level {} to level {}.", lv, lv - 1);

            // Update the back look respectively.
            seek.back_look[lv] = shortcut_taken;

            // End the loop at the last level.
            if lv == 1 {
                // We decremented the level previously, and given that our old level is one, the
                // new level is zero.

                log!(INTERNAL, "We're at the last level now.");

                break;
            }
        }

        // We're now at the bottom layer, and we need to find a match by iterating over the nodes
        // of this layer.
        if let Some(shortcut) = iter.next() {
            if let Some(found) = shortcut.node.iter().find(|x| searcher.is_match(x)) {
                // Set the seek's found node to the first match (as defined by the searcher).
                seek.node = found;
            } else {
                // No match was found, return error.
                return Err(());
            }
        } else {
            // We reached the end of iterator.
            return Err(());
        }

        seek.check();

        // Everything have been initialized, including all the back look cells (i.e. every level
        // have been visited).
        Ok(seek)
    }
}

// Here is a rare Ferris to cheer you up.
//          |
//        \ _ /
//      -= (_) =-
//        /   \         _\/_ _\/_
//          |           //o\ /o\\
//   _____ _ __ __ _______|____|___________
// =-=-_-__=_-= _=_=-=_,-'|_   |_  ()    ()
//  =- _=-=- -_=-=_,-     "-   "-   \/\/\/
//    =- =- -=.--"                  &_^^_&
//                                  \    /
// Don't share beach crab or it will lose
// its value.