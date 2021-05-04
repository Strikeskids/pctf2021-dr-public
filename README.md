# PlaidCTF 2021: dr

Not feeling well? Went to the grocery store recently? Pick up your phone and
call our office. The [doctor][./dr] will see you now.

To build this challenge yourself, run

```bash
cargo build --release
cp target/release/dr .
```

## Details

The intent of this challenge was to showcase a really neat technique called
regular expression derivatives. I discovered these from [Benn Lynn's blog][1]
and there's also a good [paper][2] about them.

To solve this challenge, you need to find the unique insurance number accepted
by the regex. I designed the regex to be too large to fully expand into a DFA,
so you need to somehow reduce the complexity.

The intended solution splits the regex into two pieces, expands the DFA to
find all the solutions to the first piece and checks those solutions against
the second piece.

### Fun combinators

Because this regex derivatives technique is so powerful, I was able to add in
several new regex combinators that one does not typically find.

The `Consider` combinator computes a mod. The special thing here is I allow
you to write the number in any base you want by providing a regex  for each
digit. 

I use this power to slightly strange effect by making  0 be `[cdb]` and 1 be
`cdb`. With these two digits, it's not possible to uniquely parse every string
into a single number. Instead, the `Consider` matches if any valid parsing has
the correct modulus.

The `Moon` combinator is a little more vanilla: it's a Kleene star plus a
finite repeat. You need to repeat the input regex `kn` times for a particular
`k` of your choice. Quirks of the implementation mean you can constrain the
minimum length by making the starting phase really large.

This implementation also supports complement or negation, where you make every
matching string not match and vice versa, and intersection,  where you match
only if all your subcomponents match. These are fairly standard when talking
about DFAs but not standard for regexes.

[1]: http://benlynn.blogspot.com/2018/06/regex-derivatives.html
[2]: https://www.ccs.neu.edu/home/turon/re-deriv.pdf
