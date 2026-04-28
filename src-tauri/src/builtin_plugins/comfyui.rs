use crate::builtin_plugins::{BuiltinPlugin, PluginConfigOption};

pub fn plugin() -> BuiltinPlugin {
    BuiltinPlugin {
        id: "comfyui-image".to_string(),
        name: "ComfyUI 生图".to_string(),
        description: "调用 ComfyUI 生成图片并推送到设备".to_string(),
        category: "AI".to_string(),
        config: vec![
            PluginConfigOption {
                name: "comfyuiUrl".to_string(),
                label: "ComfyUI 地址".to_string(),
                input_type: Some("text".to_string()),
                options: vec![],
                default: "http://127.0.0.1:8188".to_string(),
            },
            PluginConfigOption {
                name: "workflow".to_string(),
                label: "工作流 JSON".to_string(),
                input_type: Some("textarea".to_string()),
                options: vec![],
                default: "".to_string(),
            },
            PluginConfigOption {
                name: "promptNodeId".to_string(),
                label: "提示词节点 ID".to_string(),
                input_type: Some("text".to_string()),
                options: vec![],
                default: "6".to_string(),
            },
            PluginConfigOption {
                name: "promptField".to_string(),
                label: "提示词字段名".to_string(),
                input_type: Some("text".to_string()),
                options: vec![],
                default: "text".to_string(),
            },
            PluginConfigOption {
                name: "prompt".to_string(),
                label: "提示词".to_string(),
                input_type: Some("text".to_string()),
                options: vec![],
                default: "".to_string(),
            },
            PluginConfigOption {
                name: "seedNodeId".to_string(),
                label: "Seed 节点 ID".to_string(),
                input_type: Some("text".to_string()),
                options: vec![],
                default: "3".to_string(),
            },
            PluginConfigOption {
                name: "seedField".to_string(),
                label: "Seed 字段名".to_string(),
                input_type: Some("text".to_string()),
                options: vec![],
                default: "seed".to_string(),
            },
            PluginConfigOption {
                name: "randomizeSeed".to_string(),
                label: "随机 Seed".to_string(),
                input_type: Some("checkbox".to_string()),
                options: vec![],
                default: "true".to_string(),
            },
        ],
        code: r#"(async function() {
    const comfyuiUrl = config.comfyuiUrl || 'http://127.0.0.1:8188';
    const workflowStr = config.workflow;
    const promptNodeId = config.promptNodeId || '6';
    const promptField = config.promptField || 'text';
    const prompt = config.prompt;
    const seedNodeId = config.seedNodeId || '3';
    const seedField = config.seedField || 'seed';
    const randomizeSeed = config.randomizeSeed !== 'false';

    if (!workflowStr || workflowStr.trim() === '') {
        throw new Error('请先点击「配置」按钮填写工作流 JSON');
    }
    if (!prompt || prompt.trim() === '') {
        throw new Error('请输入提示词');
    }

    let workflow;
    try {
        workflow = JSON.parse(workflowStr);
    } catch (e) {
        throw new Error('工作流 JSON 格式错误: ' + e.message);
    }

    const nodeIds = Object.keys(workflow);
    if (nodeIds.length === 0) {
        throw new Error('工作流为空');
    }

    const firstNode = workflow[nodeIds[0]];
    if (!firstNode.class_type) {
        throw new Error('请使用 "Save (API Format)" 导出工作流');
    }

    if (!workflow[promptNodeId]) {
        throw new Error('找不到提示词节点 "' + promptNodeId + '"，可用节点: ' + nodeIds.join(', '));
    }
    if (!workflow[promptNodeId].inputs) {
        throw new Error('节点 ' + promptNodeId + ' 没有 inputs 字段');
    }

    workflow[promptNodeId].inputs[promptField] = prompt;

    if (randomizeSeed && workflow[seedNodeId] && workflow[seedNodeId].inputs) {
        workflow[seedNodeId].inputs[seedField] = Math.floor(Math.random() * 1000000000);
    }

    let result;
    try {
        result = await postJson(comfyuiUrl + '/prompt', JSON.stringify({ prompt: workflow }));
    } catch (e) {
        throw new Error('无法连接 ComfyUI (' + comfyuiUrl + ')');
    }

    const promptId = result.prompt_id;
    if (!promptId) {
        throw new Error('提交失败');
    }

    let outputs = null;
    let retries = 0;
    const maxRetries = 300;

    while (retries < maxRetries) {
        await sleep(1000);
        retries++;

        const history = await fetchJson(comfyuiUrl + '/history/' + promptId);

        if (history[promptId]) {
            if (history[promptId].status && history[promptId].status.status_str === 'error') {
                throw new Error('生成失败');
            }

            if (history[promptId].outputs) {
                const out = history[promptId].outputs;
                const hasImages = Object.keys(out).some(k => out[k].images && out[k].images.length > 0);
                if (hasImages) {
                    outputs = out;
                    break;
                }
            }
        }
    }

    if (!outputs) {
        throw new Error('生成超时 (' + retries + '秒)');
    }

    let imageDataUrl = null;
    for (const nodeId of Object.keys(outputs)) {
        const nodeOutput = outputs[nodeId];
        if (nodeOutput.images && nodeOutput.images.length > 0) {
            const img = nodeOutput.images[0];
            let url = comfyuiUrl + '/view?filename=' + encodeURIComponent(img.filename) + '&type=' + (img.type || 'output');
            if (img.subfolder) url += '&subfolder=' + encodeURIComponent(img.subfolder);
            imageDataUrl = await fetchBase64(url);
            break;
        }
    }

    if (!imageDataUrl) {
        throw new Error('未找到图片输出');
    }

    return { type: 'image', imageDataUrl: imageDataUrl, title: 'ComfyUI 生图' };
})()"#
            .to_string(),
        supports_loop: false,
        ..Default::default()
    }
}
