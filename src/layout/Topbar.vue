<template>
    <v-card class="header">
        <Breadcrumbs />
        <div class="mt-3 ml-9 gamepad">
            <v-icon icon="mdi-sort-variant-lock-open" />
        </div>
        <div class="head_logo ml-4 mr-1">
            <img src="" height="40" />
        </div>
        <v-btn variant="text" icon="mdi-menu">
            <v-icon size="small"></v-icon>
        </v-btn>
        <v-spacer></v-spacer>
        <div style="width: 320px" class="search_ip mr-2">
            <v-text-field
                rounded
                variant="outlined"
                density="compact"
                label="Search"
                prepend-inner-icon="mdi-magnify"
                hide-details
                clearable
            ></v-text-field>
        </div>
        <div class="tool_btns">
            <v-btn variant="text" icon="mdi-theme-light-dark" />
            <v-menu>
                <template v-slot:activator="{ props }">
                    <v-btn v-bind="props" variant="text" icon="mdi-bell-outline">
                        <v-badge content="2" color="error">
                            <v-icon size="small"></v-icon>
                        </v-badge>
                    </v-btn>
                </template>
                <v-card min-width="300" class="px-2">
                    <v-list style="height: 200px; overflow: auto">
                        <v-list-item
                            prepend-avatar=""
                            subtitle="Go out for a drink at night 🍺🍺🍺"
                            title="John 铁柱"
                        />
                    </v-list>
                    <div class="pt-2 pb-2 d-flex jsb">
                        <v-btn variant="text" color="primary"> All messages </v-btn>
                        <v-tooltip text="Clear" location="bottom">
                            <template v-slot:activator="{ props }">
                                <v-btn
                                    v-bind="props"
                                    variant="text"
                                    color="#999"
                                    style="margin-left: auto"
                                    icon="mdi-trash-can-outline"
                                    size="small"
                                >
                                    <v-icon color="red" size="small" icon="mdi-trash-can-outline" />
                                </v-btn>
                            </template>
                        </v-tooltip>
                    </div>
                </v-card>
            </v-menu>
            <v-btn variant="text" icon="mdi-github">
                <v-icon size="small"></v-icon>
            </v-btn>
            <v-btn variant="text" icon="mdi" class="mr-2">
                <v-avatar size="small" class="avatar">
                    <v-img src="" alt="陈咩啊"></v-img>
                </v-avatar>
                <v-menu activator="parent" transition="slide-y-transition">
                    <v-list nav density="compact" class="pa-2">
                        <v-list-item title="" prepend-avatar="">
                            <template #subtitle>
                                <div style="margin-top: 4px">
                                    <v-icon size="x-small" icon="mdi-email-outline"></v-icon>
                                    <span class="ml-1"></span>
                                </div>
                            </template>
                        </v-list-item>
                        <v-divider class="my-2"></v-divider>
                        <v-list-item title="Github" append-icon="mdi-github" />
                        <v-list-item title="Sign out" append-icon="mdi-login" />
                    </v-list>
                </v-menu>
            </v-btn>
            <!-- 修改设置按钮 -->
            <v-btn 
                variant="text" 
                icon="mdi-cog" 
                @click="toggleSettings"
                :color="isSettingsActive ? 'primary' : undefined"
            >
                <v-icon size="small"></v-icon>
            </v-btn>
        </div>
    </v-card>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import Breadcrumbs from '@/components/Breadcrumbs.vue';
import SettingsPanel from '@/components/SettingsPanel.vue';

const emit = defineEmits(['toggle-settings']);
const isSettingsActive = ref(false);

function toggleSettings() {
    isSettingsActive.value = !isSettingsActive.value;
    emit('toggle-settings');
}
</script>