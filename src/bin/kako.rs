use kako::Bot;
use kako::Er;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use twilight_gateway::Event;
use twilight_model::gateway::payload::{MessageCreate, Ready};
use twilight_model::id::UserId;

#[tokio::main]
async fn main() -> ! {
    use std::io::{Stderr, stderr, Write};
    let mut errp: Stderr = stderr();

    loop {
        if let Err(e) = kako_main().await {
            #[allow(unused_must_use)]
            match e.to_string().as_str() {
                "restart" => continue,
                "exit" => std::process::exit(0),
                a => errp.write_fmt(format_args!("{}\n", a)),
            };
        };
    }
}

async fn kako_main() -> Result<(), Er> {
    fn get(p: &str) -> Result<String, Er> {
        use std::fs::File;
        use std::io::Read;
        let mut file: File = File::open(p)?;
        let mut buf: Vec<u8> = Vec::new();
        file.read_to_end(&mut buf)?;
        let string: String = String::from_utf8(buf)?;
        Ok(string)
    }

    let token: String = get("token")?;
    let owner_id: u64 = get("owner")?.parse()?;

    let mut kako: Bot = Bot::connect(token, Some(UserId::from(owner_id))).await?;

    while let Ok(a) = kako.run().await {
        matcher(&mut kako, a).await?;
    }

    Ok(())
}

async fn matcher(kako: &mut Bot, event: Event) -> Result<(), Er> {
    match event {
        Event::Ready(ready) => on_ready(kako, ready),
        Event::MessageCreate(message) => commands(kako, message).await,
        _ => Ok(()),
    }
}

fn on_ready(kako: &mut Bot, ready: Box<Ready>) -> Result<(), Er> {
    use std::io::{Stdout, stdout, Write};
    let mut outp: Stdout = stdout();

    kako.set_name(ready.user.name);

    #[allow(unused_must_use)]
    outp.write_fmt(format_args!("{} ready!\n", kako));

    Ok(())

}

async fn commands(kako: &mut Bot, message: Box<MessageCreate>) -> Result<(), Er> {
    let mut vmsg: Vec<&str> = message.content.split(' ').collect();

    if message.author.bot {
        return Ok(());
    }

    if message.author.id == UserId(687492720356491400) {
        kako.create_message(message.channel_id, "shut up")
            .await?;
    }

    for m in &message.mentions {
        if m.id == UserId(687492720356491400) {
            kako.create_message(message.channel_id, format!("haha <@{}>", m.id))
                .await?;
        }
    }

    if vmsg[0].to_ascii_lowercase() != kako.name().to_ascii_lowercase() {
        return Ok(());
    }

    if vmsg.len() >= 2 {
        vmsg.remove(0);
    } else {
        return Ok(());
    }

    match vmsg[0] {
        "info" => commands::info(kako, message).await,
        "exit" => commands::exit(kako, message).await,
        "restart" => commands::restart(kako, message).await,

        a => {
            kako.create_message(
                message.channel_id,
                format!("command `{}` not recognized", a),
            )
            .await
        }
    }
}

mod commands {
    use crate::ErrOps;
    use kako::{Bot, Er};
    use twilight_model::gateway::payload::MessageCreate;

    pub async fn info(kako: &mut Bot, message: Box<MessageCreate>) -> Result<(), Er> {
        kako.create_message(
            message.channel_id,
            format!(
                "{} v{}\n",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
            ),
        )
        .await?;
        Ok(())
    }

    pub async fn restart(kako: &mut Bot, message: Box<MessageCreate>) -> Result<(), Er> {
        if let Some(owner) = kako.owner() {
            if message.author.id == owner {
                kako.create_message(message.channel_id, "restarting…")
                    .await?;
                Err(Box::new(ErrOps::Restart))
            } else {
                kako.create_message(message.channel_id, "You are not the owner")
                    .await
            }
        } else {
            kako.create_message(message.channel_id, "an owner is not registered")
                .await
        }
    }

    pub async fn exit(kako: &mut Bot, message: Box<MessageCreate>) -> Result<(), Er> {
        if let Some(owner) = kako.owner() {
            if message.author.id == owner {
                kako.create_message(message.channel_id, "exiting…").await?;
                Err(Box::new(ErrOps::Exit))
            } else {
                kako.create_message(message.channel_id, "You are not the owner")
                    .await
            }
        } else {
            kako.create_message(message.channel_id, "an owner is not registered")
                .await
        }
    }
}

#[derive(Debug)]
enum ErrOps {
    Restart,
    Exit,
}

impl Display for ErrOps {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ErrOps::Restart => "restart",
                ErrOps::Exit => "exit",
            }
        )
    }
}

impl Error for ErrOps {}
