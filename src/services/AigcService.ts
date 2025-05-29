import { SettingService } from './SettingService';
import OpenAI from 'openai';
import { listen } from '@tauri-apps/api/event';
import { invoke } from "@tauri-apps/api/core";

class AigcService {
    static {
        // 监听来自 Rust 的标签生成请求
        listen('aigc:generate-tags', async (event: { payload: [string, string[]] }) => {
            try {
                const [content, availableTags] = event.payload;
                console.log('收到标签生成请求:', { content, availableTags });
                
                const tags = await AigcService.generateTags(content, availableTags);
                console.log('生成的标签:', tags);
                
                if (tags) {
                    // 调用 Rust 命令返回生成的标签
                    await invoke('receive_generated_tags', { tags });
                }
            } catch (error) {
                console.error('处理标签生成请求失败:', error);
            }
        });
    }
    private static getOpenAIClient(apiKey: string): OpenAI {
        return new OpenAI({
            baseURL: 'https://api.deepseek.com',
            apiKey: apiKey,
            dangerouslyAllowBrowser: true  // 允许在浏览器环境中使用
        });
    }

    /**
     * 调用AI API生成内容摘要
     * @param content 原始内容
     * @returns 生成的摘要
     */
    static async generateSummary(content: string): Promise<string | null> {
        try {
            const settings = SettingService.getAigcSettings();

            // 检查AI功能是否启用
            if (!settings || !settings.switch) {
                console.log('AI功能未启用');
                return null;
            }

            // 获取API配置
            const apiToken = settings.tokens;

            if (!apiToken) {
                console.error('未配置API Token');
                return null;
            }

            const openai = this.getOpenAIClient(apiToken);

            // 构造摘要提示词
            const completion = await openai.chat.completions.create({
                messages: [
                    {
                        role: "system",
                        content: "你是一个专业的内容总结助手，善于提取关键信息并生成简洁的总结。"
                    },
                    {
                        role: "user",
                        content: `请对以下内容进行简洁的总结，提取最重要的信息点，总结不超过50个字：\n\n${content}`
                    }
                ],
                model: "deepseek-chat",
            });

            return completion.choices[0].message.content;
        } catch (error) {
            console.error('生成摘要失败:', error);
            return null;
        }
    }

    /**
     * 调用AI API生成内容改进建议
     * @param content 原始内容
     * @returns 生成的改进建议
     */
    static async generateImprovement(content: string): Promise<string | null> {
        try {
            const settings = SettingService.getAigcSettings();

            // 检查AI功能是否启用
            if (!settings || !settings.switch) {
                console.log('AI功能未启用');
                return null;
            }

            // 获取API配置
            const apiToken = settings.tokens;

            if (!apiToken) {
                console.error('未配置API Token');
                return null;
            }

            const openai = this.getOpenAIClient(apiToken);

            // 构造内容改进提示词
            const completion = await openai.chat.completions.create({
                messages: [
                    {
                        role: "system",
                        content: "你是一个专业的待办事项优化助手，善于分析内容并提供具体改进建议。"
                    },
                    {
                        role: "user",
                        content: `
            请对以下待办事项内容进行分析并提供改进建议。分析内容的清晰度、完整性和可操作性，并提出具体的改进建议，以使其更加明确、结构化和易于执行。
            
            原始内容:
            ${content}
            
            请提供3-5点具体的改进建议，每条建议应简洁明了，并解释为什么这样改进会使待办事项更有效。`
                    }
                ],
                model: "deepseek-chat",
            });

            return completion.choices[0].message.content;
        } catch (error) {
            console.error('生成改进建议失败:', error);
            return null;
        }
    }

    /**
     * 调用AI API为内容生成标签
     * @param content 原始内容
     * @param availableTags 可选的标签列表
     * @returns 生成的标签列表
     */
    static async generateTags(content: string, availableTags: string[]): Promise<string[] | null> {
        try {
            const settings = SettingService.getAigcSettings();
            console.log('生成标签中', content, availableTags);
            // 检查AI功能是否启用
            if (!settings || !settings.switch) {
                console.log('AI功能未启用');
                return null;
            }

            // 获取API配置
            const apiToken = settings.tokens;

            if (!apiToken) {
                console.error('未配置API Token');
                return null;
            }

            const openai = this.getOpenAIClient(apiToken);

            // 构造标签生成提示词
            const completion = await openai.chat.completions.create({
                messages: [
                    {
                        role: "system",
                        content: "你是一个专业的内容分类助手，善于为内容选择合适的标签。"
                    },
                    {
                        role: "user",
                        content: `
            为以下内容选择合适的标签。标签可以不止一个，但只能从给出的标签列表中选择。
            
            可选标签列表:
            ${availableTags.join(',')}
            
            需要打标签的内容:
            ${content}
            
            请只返回你选择的标签,多个标签之间用英文逗号分隔。`
                    }
                ],
                model: "deepseek-chat",
            });

            const tags = completion.choices[0].message.content?.trim().split(',').filter(tag => tag.length > 0) || [];
            return tags;

        } catch (error) {
            console.error('生成标签失败:', error);
            return null;
        }
    }
}

export default AigcService;