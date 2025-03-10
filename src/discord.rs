use serenity::all::{Guild, GuildChannel, Context, GuildId};

pub fn find_guild_channel(guilds: Vec<Guild>, guild_name: String, channel_name: String) -> Vec<GuildChannel> {
    let mut result: Vec<GuildChannel> = Vec::new();

    for guild in guilds {
        if guild.name == guild_name {
            for channel in guild.channels.values() {
                if channel.name == channel_name {
                    result.push(channel.clone());
                }
            }
        }
    }

    return result;
}

pub fn list_guilds(ctx: Context, ids: Vec<GuildId>) -> Vec<Guild> {
    let mut guilds: Vec<Guild> = Vec::new();

    for id in ids {
        let guild = ctx.cache.guild(id).unwrap();
        guilds.push(guild.clone());
    }

    return guilds;
}

#[cfg(test)]
mod tests {
    use super::*;
    use serenity::all::ChannelId;

    fn create_guild(name: String, channels: Vec<GuildChannel>) -> Guild {
        let mut guild = Guild::default();
        guild.name = name;

        for channel in channels {
            guild.channels.insert(channel.id, channel);
        }

        return guild;
    }

    fn create_channel(name: String, id: ChannelId) -> GuildChannel {
        let mut channel = GuildChannel::default();
        channel.name = name;
        channel.id = id;
        return channel;
    }

    #[test]
    fn empty_guilds() {
        let guilds: Vec<Guild> = Vec::new(); 
        let got = find_guild_channel(guilds, String::new(), String::new());
        assert_eq!(0, got.len())
    }

    #[test]
    fn matching_guild_channel() {
        let mut guilds: Vec<Guild> = Vec::new(); 
        let guild_name: String = "my guild".to_string();
        let channel_name: String = "my channel".to_string();

        guilds.push(
            create_guild(
                guild_name.clone(),
                vec![
                    create_channel(channel_name.clone(), ChannelId::new(1)),
                ]
            )
        );

        guilds.push(
            create_guild(
                "test".to_string(),
                vec![
                    create_channel("another channel".to_string(), ChannelId::new(3)),
                ]
            )
        );

        let got = find_guild_channel(guilds, guild_name, channel_name);

        assert_eq!(1, got.len())
    }

    #[test]
    fn non_matching_guild() {
        let mut guilds: Vec<Guild> = Vec::new(); 
        let guild_name: String = "my guild".to_string();
        let channel_name: String = "my channel".to_string();

        guilds.push(
            create_guild(
                "test".to_string(),
                vec![]
            )
        );

        guilds.push(
            create_guild(
                "test 2".to_string(),
                vec![]
            )
        );

        let got = find_guild_channel(guilds, guild_name, channel_name);

        assert_eq!(0, got.len())
    }
}
