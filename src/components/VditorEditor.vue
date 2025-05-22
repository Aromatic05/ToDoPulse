<template>
    <div :id="editorId" ref="vditorRefElement"></div>
</template>

<script lang="ts">
import { defineComponent, ref, watch, onMounted, onBeforeUnmount, nextTick} from 'vue';
import Vditor from 'vditor';
import 'vditor/dist/index.css';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { dataDir, join } from '@tauri-apps/api/path';

// 扩展 Vditor 类型定义
declare module 'vditor' {
    interface IVditor {
        destroy: () => void;
        getValue: () => string;
        setValue: (content: string) => void;
        insertValue: (value: string, render?: boolean) => void; // 确保此方法存在
    }
}

export default defineComponent({
    name: 'VditorEditor',
    props: {
        modelValue: { // 用于 v-model 同步内容
            type: String,
            default: ''
        },
        eventId: { // 用于文件上传
            type: String,
            required: true
        },
        cardTitle: { // 用于构建动态链接基础
            type: String,
            default: 'default_event' // 提供一个默认值以防万一
        },
        editorId: { // 允许动态ID，以防页面上有多个编辑器实例 (虽然通常一个模态框一个)
            type: String,
            default: 'vditor-editor-instance'
        }
    },
    emits: ['update:modelValue', 'initialized'], // 'initialized' 事件用于通知父组件编辑器已准备好
    setup(props, { emit }) {
        const vditorInstance = ref<Vditor | null>(null);
        const vditorRefElement = ref<HTMLElement | null>(null); // 用于挂载 Vditor
        const isInitialized = ref(false);

        const isDarkMode = (): boolean => {
            return document.body.classList.contains('dark') || document.body.classList.toString().includes('-dark');
        };

        const getDynamicLinkBase = async () => {
            if (!props.cardTitle) {
                console.warn("cardTitle is not available for getDynamicLinkBase, using empty string.");
                return '';
            }
            try {
                const dataDirPath = await dataDir();
                // 使用 props.cardTitle, 如果为空则使用默认值
                const safeCardTitle = props.cardTitle || 'default_event_files';
                const targetDirectoryPath = await join(dataDirPath, 'ToDoPulse' , safeCardTitle);
                let linkBaseUrl = convertFileSrc(targetDirectoryPath);
                if (!linkBaseUrl.endsWith('/')) {
                    linkBaseUrl += '/';
                }
                console.log('VditorEditor: Dynamically constructed linkBase:', linkBaseUrl);
                return linkBaseUrl;
            } catch (error) {
                console.error("VditorEditor: Error constructing dynamic linkBase:", error);
                return '';
            }
        };

        const initEditor = async () => {
            if (!vditorRefElement.value) {
                console.error('Vditor mount element not found.');
                return;
            }

            const dynamicLinkBase = await getDynamicLinkBase();
            await nextTick();

            if (vditorInstance.value) {
                try {
                    vditorInstance.value.destroy();
                } catch (e) {
                    console.warn('Error destroying previous Vditor instance:', e);
                }
                vditorInstance.value = null;
            }

            vditorInstance.value = new Vditor(vditorRefElement.value, {
                height: 500,
                width: '100%',
                theme: isDarkMode() ? 'dark' : 'classic',
                toolbarConfig: { pin: true },
                cache: { enable: false },
                placeholder: '请输入内容...',
                mode: 'wysiwyg', // 或者 'ir', 'sv'
                preview: {
                    markdown: {
                        linkBase: dynamicLinkBase,
                    }
                },
                after: () => {
                    isInitialized.value = true;
                    if (props.modelValue) {
                        vditorInstance.value?.setValue(props.modelValue);
                    }
                    emit('initialized', vditorInstance.value); // 通知父组件
                },
                input: (value: string) => {
                    emit('update:modelValue', value);
                },
                upload: {
                    url: '/upload-handler', // 占位
                    max: 10 * 1024 * 1024,
                    accept: 'image/*, .pdf, .docx, .xlsx, .zip',
                    multiple: true,
                    fieldName: 'file',
                    // success 和 handler 的逻辑与原组件相似，但注意 props.eventId 的使用
                    handler: async (files: File[]) => {
                        if (!props.eventId) {
                            return '事件ID无效，无法上传文件';
                        }

                        try {
                            const eventId = props.eventId;
                            const promises = files.map(async (file) => {
                                try {
                                    const arrayBuffer = await file.arrayBuffer();
                                    const uint8Array = new Uint8Array(arrayBuffer);
                                    let base64 = '';
                                    const chunkSize = 524288;
                                    for (let i = 0; i < uint8Array.length; i += chunkSize) {
                                        const chunk = uint8Array.slice(i, i + chunkSize);
                                        base64 += String.fromCharCode.apply(null, Array.from(chunk));
                                    }
                                    const base64Data = btoa(base64);

                                    const result = await invoke('upload_file', {
                                        filename: file.name,
                                        filedata: base64Data,
                                        eventid: eventId
                                    });
                                    return result;
                                } catch (err) {
                                    console.error('文件上传失败:', err);
                                    return { code: 1, msg: `上传失败: ${err}`, data: { errFiles: [file.name], succMap: {} } };
                                }
                            });

                            const results = await Promise.all(promises);
                            const combinedResult = { code: 0, msg: '上传成功', data: { errFiles: [] as string[], succMap: {} as Record<string, string> } };

                            results.forEach((result: any) => {
                                try {
                                    const parsedResult = typeof result === 'string' ? JSON.parse(result) : result;
                                    if (parsedResult.code === 0 && parsedResult.data) {
                                        if (parsedResult.data.succ_map) {
                                            Object.entries(parsedResult.data.succ_map).forEach(([filename, url]) => {
                                                combinedResult.data.succMap[filename] = `![${filename}](${url})`;
                                            });
                                        }
                                        if (parsedResult.data.err_files && parsedResult.data.err_files.length > 0) {
                                            combinedResult.data.errFiles.push(...parsedResult.data.err_files);
                                        }
                                    } else {
                                        combinedResult.data.errFiles.push(parsedResult.msg || '上传失败，未知错误');
                                    }
                                } catch (e) {
                                    console.error('解析上传结果失败:', e, result);
                                    combinedResult.data.errFiles.push('上传结果解析失败');
                                }
                            });

                            if (combinedResult.code === 0 && combinedResult.data.succMap) {
                                Object.values(combinedResult.data.succMap).forEach((markdownLink) => {
                                    vditorInstance.value?.insertValue(markdownLink + '\n');
                                });
                            }
                            return JSON.stringify(combinedResult); // Vditor期望的是字符串
                        } catch (error) {
                            console.error('文件上传处理器错误:', error);
                            return `上传失败: ${error}`;
                        }
                    },
                    linkToImgFormat: (responseText) => { // 处理剪贴板图片上传 (网络图片地址)
                        try {
                            const result = JSON.parse(responseText);
                            if (result.code === 0 && result.data && result.data.url) {
                                return JSON.stringify({
                                    code: 0,
                                    data: {
                                        errFiles: [],
                                        succMap: {
                                            [result.data.originalURL]: result.data.url // 假设 originalURL 存在
                                        }
                                    }
                                });
                            }
                        } catch (e) {
                            console.error('解析上传响应失败', e);
                        }
                        return responseText;
                    },
                    linkToImgCallback: async (responseText) => { // 处理剪贴板中的图片URL
                         try {
                            const urlData = JSON.parse(responseText); // Vditor 内部会把 file[0].name (即URL) 包装成 { url: "..." } 再传给这个回调
                            const url = urlData.url;

                            if (!props.eventId) {
                                console.error('保存远程图片失败: 没有事件ID');
                                return '';
                            }
                            const result = await invoke('save_remote_image', {
                                url: url,
                                event_id: props.eventId
                            });

                            const resultData = result as any;
                            if (resultData.code === 0 && resultData.data && resultData.data.url) {
                                return resultData.data.url; // 返回的是 Markdown 图片链接或纯 URL，Vditor 会处理
                            } else {
                                console.error('保存远程图片失败:', resultData);
                                return '';
                            }
                        } catch (e) {
                            console.error('处理远程图片失败:', e);
                            return '';
                        }
                    }
                }
            });
        };

        onMounted(() => {
            // 延迟初始化，确保父组件的 modelValue 和 eventId 已正确传递
            // 并且 DOM 元素已准备好
             nextTick().then(() => {
                if (props.eventId) { // 确保关键 prop 已就绪
                    initEditor();
                } else {
                    console.warn("VditorEditor: eventId not available on mount, delaying init.");
                }
            });
        });

        onBeforeUnmount(() => {
            if (vditorInstance.value) {
                try {
                    vditorInstance.value.destroy();
                } catch (e) {
                    console.warn('Error destroying Vditor instance on unmount:', e);
                }
                vditorInstance.value = null;
            }
        });

        watch(() => props.modelValue, (newValue) => {
            if (vditorInstance.value && isInitialized.value && vditorInstance.value.getValue() !== newValue) {
                vditorInstance.value.setValue(newValue);
            }
        });

        watch(() => props.eventId, (newId, oldId) => {
            if (newId && newId !== oldId && vditorRefElement.value) {
                // EventId 变化，可能需要重新初始化或至少更新上传配置，这里选择重新初始化
                console.log("VditorEditor: eventId changed, re-initializing editor.");
                initEditor();
            } else if (newId && !vditorInstance.value && vditorRefElement.value) {
                // 如果编辑器因 eventId 初始缺失而未初始化，现在初始化它
                console.log("VditorEditor: eventId became available, initializing editor.");
                initEditor();
            }
        });

        // 暴露一个方法给父组件，用于获取当前编辑器的值 (如果需要)
        const getValue = () => {
            return vditorInstance.value?.getValue() || '';
        };

        // 暴露一个方法给父组件，用于设置编辑器的值 (如果需要)
        const setValue = (content: string) => {
            vditorInstance.value?.setValue(content);
        }

        return {
            vditorRefElement,
            getValue,
            setValue,
            // 不需要返回 vditorInstance 本身给 template
        };
    }
});
</script>

<style scoped>
@import '@/styles/vditor.css';
.vditor-editor-instance {
    min-height: 400px; /* 确保编辑器有一个最小高度 */
}
</style>