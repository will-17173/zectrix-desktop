use crate::builtin_plugins::{BuiltinPlugin, PluginConfigOption};

pub fn plugin() -> BuiltinPlugin {
    BuiltinPlugin {
        id: "bilibili-uploader-info".to_string(),
        name: "B站UP主信息".to_string(),
        description: "获取B站UP主的账号信息、粉丝数据、播放统计和最新视频".to_string(),
        category: "社交".to_string(),
        config: vec![PluginConfigOption {
            name: "userId".to_string(),
            label: "用户ID".to_string(),
            input_type: Some("text".to_string()),
            options: vec![],
            default: "".to_string(),
        }],
        code: r#"(async function() {
    const userId = config.userId;
    if (!userId) throw new Error('请输入用户ID');

    const headers = {
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
        'Referer': 'https://www.bilibili.com/'
    };

    const cardInfo = await fetchJsonWithHeaders('https://api.bilibili.com/x/web-interface/card?mid=' + userId + '&photo=true', headers);
    if (cardInfo.code !== 0) throw new Error('获取用户信息失败: ' + cardInfo.message);
    const name = cardInfo.data.card.name;
    const mid = cardInfo.data.card.mid;
    const sign = cardInfo.data.card.sign || '暂无签名';

    await sleep(2000);

    const relationStat = await fetchJsonWithHeaders('https://api.bilibili.com/x/relation/stat?vmid=' + userId, headers);
    const following = relationStat.data?.following || 0;
    const follower = relationStat.data?.follower || 0;

    await sleep(2000);

    const upstat = await fetchJsonWithHeaders('https://api.bilibili.com/x/space/upstat?mid=' + userId, headers);
    const archiveView = upstat.data?.archive?.view || 0;
    const likes = upstat.data?.likes || 0;

    await sleep(2000);

    const arcSearch = await fetchJsonWithHeaders('https://api.bilibili.com/x/space/arc/search?mid=' + userId + '&pn=1&ps=20', headers);
    if (arcSearch.code !== 0) {
        const text = name + '(' + mid + ')' + '\n' +
            sign + '\n' +
            '关注: ' + following + ' 粉丝: ' + follower + '\n' +
            '总播放: ' + archiveView + ' 总点赞: ' + likes + '\n' +
            '最新视频: 暂无数据';
        return { type: 'text', text: text, title: 'B站UP主信息', fontSize: 20 };
    }
    const vlist = arcSearch.data?.list?.vlist || [];
    let latestVideoText = '暂无视频';
    if (vlist.length > 0) {
        const latestVideo = vlist[0];
        latestVideoText = latestVideo.title + '\n播放: ' + latestVideo.play + ' 评论: ' + latestVideo.comment;
    }

    const text = name + '(' + mid + ')' + '\n' +
        sign + '\n' +
        '关注: ' + following + ' 粉丝: ' + follower + '\n' +
        '总播放: ' + archiveView + ' 总点赞: ' + likes + '\n' +
        '最新视频:\n' + latestVideoText;

    return { type: 'text', text: text, title: 'B站UP主信息', fontSize: 20 };
})()"#
            .to_string(),
        ..Default::default()
    }
}
