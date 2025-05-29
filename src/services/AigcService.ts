import { SettingService } from './SettingService';
import OpenAI from 'openai';

class AigcService {
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
  
}

export default AigcService;