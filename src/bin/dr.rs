use dr::{cheese, compile, consider, sundae, toppings, EPS, NUL};
use sha3::{Digest, Sha3_256};
use std::io::Write;

const FLAG: &[u8] = &[
    142, 88, 146, 56, 188, 95, 37, 15, 3, 163, 242, 157, 211, 208, 75, 216, 94, 121, 29, 238, 174,
    129, 111, 62, 255, 160, 75, 167, 11, 43, 214, 160, 134, 117, 143, 38, 244, 109, 87, 39, 3, 133,
];

fn try_main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    print!(
        "\
        Hello! Welcome to your remote doctor's office.\n\
        How can I help you today?\n\
        [1]: Schedule an appointment\n\
        [2]: Cancel a previous appointment\n\
        [3]: Check in\n\
        \
        > "
    );

    std::io::stdout().flush()?;
    input.clear();
    std::io::stdin().read_line(&mut input)?;
    input.pop();
    match input.parse()? {
        1 => {
            println!("Sorry, we don't have any appointments available.");
            return Ok(());
        }
        2 => {
            println!(
                "\
        There will be a $31,137 fee to cancel. It doesn't seem like you have\n\
        enough money to do that."
            );
            return Ok(());
        }
        3 => (),
        _ => return Err(Box::from("Bad choice")),
    }

    print!("What symptoms are you experiencing?\n> ");
    std::io::stdout().flush()?;
    input.clear();
    std::io::stdin().read_line(&mut input)?;
    input.pop();

    let flu = '\u{1f927}';
    let stomach_bug = '\u{1f92e}';
    let broken_leg = '\u{1f9b5}';

    let mut matcher = compile(
        ((toppings(",").fickle()
            * toppings(" ").star()
            * (sundae("vomiting") | sundae("diarrhea") | sundae("stomach pain")))
        .sun()
            * cheese(stomach_bug))
            | ((toppings(",").fickle()
                * toppings(" ").star()
                * (sundae("cannot walk") | sundae("cannot stand") | sundae("visible bone")))
            .sun()
                * cheese(broken_leg))
            | ((toppings(",").fickle()
                * toppings(" ").star()
                * (sundae("sore throat") | sundae("runny nose") | sundae("cough")))
            .sun()
                * cheese(flu)),
    );

    let (message, cost) = 'found: loop {
        for (diagnosis, response, cost) in [
            (
                stomach_bug,
                "a stomach bug. Stay home and make sure you have a bucket near your bed.",
                133337,
            ),
            (flu, "the flu. Stay home and get some bedrest.", 31337),
            (
                broken_leg,
                "a broken leg. I'd recommend not walking around.",
                313337,
            ),
        ]
        .iter()
        {
            input.push(*diagnosis);
            let diagnosed = matcher.matches(&input);
            input.pop();
            if diagnosed {
                break 'found (*response, *cost);
            }
        }

        break 'found ("a bone spur. Let me schedule you an MRI.", 73331);
    };

    print!(
        "\
    It looks like you have {}\n\
    Here's some ibuprofen to help with the pain.\n\
    \n\
    Your total is ${}.00\n\
    Do you have insurance?\n\
    > ",
        message, cost
    );
    std::io::stdout().flush()?;
    input.clear();
    std::io::stdin().read_line(&mut input)?;
    input.pop();

    if input != "yes" {
        println!(
            "\
        You don't have enough money to pay for that. I guess you'll have to\n\
        go into debt."
        );
        return Ok(());
    }

    let mut matcher = compile(
        NUL.neg()
            & (NUL.neg()
                * consider(
                    vec![toppings("cdb"), sundae("cdb") * toppings("db").star().neg()],
                    2,
                    3,
                ))
            & (NUL.neg() * cheese('1'..='3').sun() * cheese('3'..='7').sun() * NUL.neg())
            & consider(
                vec![
                    toppings("05a"),
                    toppings("16b"),
                    toppings("27cf"),
                    toppings("38dx"),
                    toppings("49e"),
                ],
                0,
                7,
            )
            & (cheese('0'..='9').moon_phase(1, 3) * cheese('a'..='f').moon_phase(2, 3))
                .moon_phase(0, 2)
            & (sundae("10") * EPS.neg()).moon_phase(0, 3)
            & (NUL.neg()
                * (sundae("af")
                    | sundae("73")
                    | cheese('0'..='9') * sundae("a")
                    | sundae("ccc")
                    | cheese('0'..='9').fan(7))
                * NUL.neg())
            .neg()
            & (NUL.neg() * (cheese('a'..='f') * NUL.neg()).fan(6)).neg()
            & (consider(
                std::iter::successors(Some('0'), |x| match x {
                    '0'..='8' | 'a'..='e' => std::char::from_u32(*x as u32 + 1),
                    '9' => Some('a'),
                    _ => None,
                })
                .map(cheese)
                .collect::<Vec<_>>(),
                7777,
                cost,
            ))
            & NUL.neg(),
    );
    loop {
        print!("What is your insurance policy number?\n> ");
        std::io::stdout().flush()?;
        input.clear();
        std::io::stdin().read_line(&mut input)?;
        input.pop();

        if matcher.matches(&input) {
            break;
        }

        println!("Your insurance card was not accepted. Please try again.")
    }

    let mut hasher = Sha3_256::new();
    hasher.update(input.as_bytes());
    let hash = hasher.finalize();

    let mut bytes = Vec::new();
    for (a, b) in hash.iter().cycle().zip(FLAG.iter()) {
        bytes.push(a ^ b);
    }
    println!(
        "\
        Now you're only going to have to pay a $13.37 copay. Here's a lollipop on\n\
        your way out:\n\
        \n\
                PCTF{{{}}}",
        String::from_utf8(bytes)?
    );

    Ok(())
}

fn main() {
    match try_main() {
        Ok(()) => (),
        Err(e) => println!(
            "\
            Sorry, we couldn't understand you.\n\
            {}\n\
            Please speak more clearly next time.",
            e
        ),
    }
}
