<template>
    <div :id="editorId" ref="vditorRefElement"></div>
</template>

<script lang="ts">
import { defineComponent, ref, watch, onMounted, onBeforeUnmount, nextTick } from 'vue';
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
        insertValue: (value: string, render?: boolean) => void;
    }
}

export default defineComponent({
    name: 'VditorEditor',
    props: {
        modelValue: {
            type: String,
            default: ''
        },
        eventId: {
            type: String,
            required: true
        },
        cardTitle: {
            type: String,
            default: 'default_event'
        },
        editorId: {
            type: String,
            default: 'vditor-editor-instance'
        }
    },
    emits: ['update:modelValue', 'initialized'],
    setup(props, { emit }) {
        const vditorInstance = ref<Vditor | null>(null);
        const vditorRefElement = ref<HTMLElement | null>(null);
        const isInitialized = ref(false);

        // 检测是否为移动设备
        const isMobileDevice = () => {
            return window.innerWidth <= 768;
        };

        const isDarkMode = (): boolean => {
            return document.body.classList.contains('dark') || document.body.classList.toString().includes('-dark');
        };

        const getDynamicLinkBase = async () => {
            // 原有代码保持不变
            if (!props.cardTitle) {
                console.warn("cardTitle is not available for getDynamicLinkBase, using empty string.");
                return '';
            }
            try {
                const dataDirPath = await dataDir();
                const safeCardTitle = props.cardTitle || 'default_event_files';
                const targetDirectoryPath = await join(dataDirPath, 'ToDoPulse', safeCardTitle);
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

            // 根据设备类型选择不同的工具栏配置
            const isMobile = isMobileDevice();

            // 移动端精简工具栏
            const mobileToolbar = [
                'headings', '|',
                'bold', 'italic', '|',
                'list', 'ordered-list', '|',
                'undo', 'redo',
            ];

            // 桌面端完整工具栏
            const desktopToolbar = [
                'emoji', 'headings', 'bold', 'italic', 'strike', '|',
                'line', 'quote', 'list', 'ordered-list', 'check', 'outdent', 'indent', '|',
                'code', 'inline-code', '|',
                'upload', 'link', 'table', '|',
                'undo', 'redo', '|',
                'edit-mode', 'both', 'preview', 'outline', '|',
                'fullscreen', 'help'
            ];

            vditorInstance.value = new Vditor(vditorRefElement.value, {
                height: isMobile ? 300 : 500, // 移动端减小高度
                width: '100%',
                theme: isDarkMode() ? 'dark' : 'classic',
                toolbar: isMobile ? mobileToolbar : desktopToolbar, // 根据设备类型选择工具栏
                toolbarConfig: {
                    pin: !isMobile, // 移动端不固定工具栏
                },
                cache: { enable: false },
                placeholder: '请输入内容...',
                mode: 'wysiwyg', // 移动端默认使用所见即所得模式
                preview: {
                    markdown: {
                        linkBase: dynamicLinkBase,
                    }
                },
                // 移动端优化设置
                comment: { enable: false },  // 移动端禁用评论功能
                typewriterMode: false,       // 移动端禁用打字机模式
                tab: '    ',                 // 简化缩进
                // 移动端调整内容和行高
                counter: {
                    enable: !isMobile,       // 移动端禁用计数器
                },
                after: () => {
                    isInitialized.value = true;
                    if (props.modelValue) {
                        vditorInstance.value?.setValue(props.modelValue);
                    }

                    // 移动端优化：添加触摸相关的类
                    if (isMobile) {
                        const editorElement = vditorRefElement.value;
                        if (editorElement) {
                            editorElement.classList.add('mobile-editor');
                        }
                    }

                    emit('initialized', vditorInstance.value);
                },
                input: (value: string) => {
                    emit('update:modelValue', value);
                },
                upload: {
                    // 原有上传配置保持不变
                    url: '/upload-handler',
                    max: 10 * 1024 * 1024,
                    accept: 'image/*, .pdf, .docx, .xlsx, .zip',
                    multiple: true,
                    fieldName: 'file',
                    handler: async (files: File[]) => {
                        // 原有处理器代码保持不变
                        if (!props.eventId) {
                            return '事件ID无效，无法上传文件';
                        }

                        try {
                            const eventId = props.eventId;
                            const promises = files.map(async (file) => {
                                // 原有上传处理代码...
                                try {
                                    const arrayBuffer = await file.arrayBuffer();
                                    const uint8Array = new Uint8Array(arrayBuffer);
                                    let base64 = '';
                                    const chunkSize = 52428;
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

                            // 原有结果处理代码...
                            const results = await Promise.all(promises);
                            const combinedResult = { code: 0, msg: '上传成功', data: { errFiles: [] as string[], succMap: {} as Record<string, string> } };

                            results.forEach((result: any) => {
                                // 原有处理逻辑...
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
                            return JSON.stringify(combinedResult);
                        } catch (error) {
                            console.error('文件上传处理器错误:', error);
                            return `上传失败: ${error}`;
                        }
                    },
                    linkToImgFormat: (responseText) => {
                        // 原有代码保持不变...
                        try {
                            const result = JSON.parse(responseText);
                            if (result.code === 0 && result.data && result.data.url) {
                                return JSON.stringify({
                                    code: 0,
                                    data: {
                                        errFiles: [],
                                        succMap: {
                                            [result.data.originalURL]: result.data.url
                                        }
                                    }
                                });
                            }
                        } catch (e) {
                            console.error('解析上传响应失败', e);
                        }
                        return responseText;
                    },
                    linkToImgCallback: async (responseText) => {
                        // 原有代码保持不变...
                        try {
                            const urlData = JSON.parse(responseText);
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
                                return resultData.data.url;
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

        // 其余生命周期和 watch 代码保持不变
        onMounted(() => {
            nextTick().then(() => {
                if (props.eventId) {
                    initEditor();
                } else {
                    console.warn("VditorEditor: eventId not available on mount, delaying init.");
                }
            });

            // 添加窗口大小变化监听，以便在设备类型改变时重新初始化编辑器
            window.addEventListener('resize', handleResize);
        });

        // 处理窗口大小变化
        const handleResize = () => {
            // 不频繁重新初始化，使用防抖处理
            if (resizeTimeout.value) clearTimeout(resizeTimeout.value);
            resizeTimeout.value = setTimeout(() => {
                if (props.eventId) {
                    initEditor();
                }
            }, 500);
        };

        const resizeTimeout = ref<any>(null);

        onBeforeUnmount(() => {
            if (vditorInstance.value) {
                try {
                    vditorInstance.value.destroy();
                } catch (e) {
                    console.warn('Error destroying Vditor instance on unmount:', e);
                }
                vditorInstance.value = null;
            }

            // 移除事件监听器
            window.removeEventListener('resize', handleResize);
            if (resizeTimeout.value) clearTimeout(resizeTimeout.value);
        });

        watch(() => props.modelValue, (newValue) => {
            if (vditorInstance.value && isInitialized.value && vditorInstance.value.getValue() !== newValue) {
                vditorInstance.value.setValue(newValue);
            }
        });

        watch(() => props.eventId, (newId, oldId) => {
            if (newId && newId !== oldId && vditorRefElement.value) {
                console.log("VditorEditor: eventId changed, re-initializing editor.");
                initEditor();
            } else if (newId && !vditorInstance.value && vditorRefElement.value) {
                console.log("VditorEditor: eventId became available, initializing editor.");
                initEditor();
            }
        });

        // 暴露方法给父组件
        const getValue = () => {
            return vditorInstance.value?.getValue() || '';
        };

        const setValue = (content: string) => {
            vditorInstance.value?.setValue(content);
        }

        return {
            vditorRefElement,
            getValue,
            setValue,
        };
    }
});
</script>

<style scoped>
@import '@/styles/vditor.css';

.vditor-editor-instance {
    min-height: 400px;
}

/* 移动设备优化样式 */
@media (max-width: 768px) {
    :deep(.vditor) {
        border-radius: 8px;
    }

    :deep(.vditor-toolbar) {
        padding: 6px 5px;
        overflow-x: auto;
        justify-content: flex-start;
        flex-wrap: nowrap;
    }

    :deep(.vditor-toolbar__item) {
        height: 28px;
        width: 28px;
        margin: 0 2px;
    }

    :deep(.vditor-toolbar__item svg) {
        height: 16px;
        width: 16px;
    }

    /* 增大触摸区域 */
    :deep(.vditor-toolbar__item button) {
        padding: 6px;
    }

    /* 调整编辑区域 */
    :deep(.vditor-ir, .vditor-wysiwyg, .vditor-sv) {
        padding: 8px 12px;
    }

    /* 减小编辑区域的默认最小高度 */
    .vditor-editor-instance {
        min-height: 250px;
    }
}

/* 移动端编辑器特殊样式 */
.mobile-editor :deep(.vditor-ir) {
    font-size: 16px;
    /* 移动端更大的字体 */
    line-height: 1.5;
}

.mobile-editor :deep(.vditor-wysiwyg) {
    font-size: 16px;
    line-height: 1.5;
}

.mobile-editor :deep(.vditor-reset) {
    font-size: 16px;
}

/* 增强触摸反馈 */
.mobile-editor :deep(.vditor-toolbar__item:active) {
    background-color: rgba(0, 0, 0, 0.1);
    transform: scale(0.96);
}
</style>