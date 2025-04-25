<template>
    <v-navigation-drawer
        v-model="drawer"
        class="my-4 layout_navigation"
        :rail="railModel"
        permanent
        rail-width="77"
        style="position: fixed"
        @click="expandDrawer"
    >
        <v-list class="py-4 mx-2 logo" nav>
            <v-list-item rounded class="mx-1">
                <template v-slot:prepend>
                    <v-btn
                        variant="text"
                        :icon="railModel ? 'mdi-arrow-expand-right' : 'mdi-arrow-collapse-left'"
                        size="small"
                        @click.stop="toggleRail"
                    ></v-btn>
                </template>
                <v-list-item-title class="title">ToDoPulse</v-list-item-title>
                <v-list-item-subtitle>Task Management</v-list-item-subtitle>
            </v-list-item>
        </v-list>
        <v-divider class="mx-5"></v-divider>

        <v-list nav class="mx-2" color="primary">
            <!-- Main Navigation Items -->
            <v-list-item
                prepend-icon="mdi-view-dashboard"
                title="Dashboard"
                class="mx-1"
                active-class="nav_active"
                rounded="lg"
            ></v-list-item>
            
            <!-- Example of a list group -->
            <v-list-group>
                <template v-slot:activator="{ props }">
                    <v-list-item
                        v-bind="props"
                        prepend-icon="mdi-format-list-checks"
                        title="Tasks"
                        rounded="lg"
                    />
                </template>
                <v-list-item
                    prepend-icon="mdi-clipboard-list"
                    title="All Tasks"
                    class="mx-1"
                    rounded="lg"
                ></v-list-item>
                <v-list-item
                    prepend-icon="mdi-star"
                    title="Important"
                    class="mx-1"
                    rounded="lg"
                ></v-list-item>
            </v-list-group>
        </v-list>
    </v-navigation-drawer>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

const props = defineProps({
  rail: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['update:rail'])

const drawer = ref(true)
const railModel = computed({
  get: () => props.rail,
  set: (value) => {
    emit('update:rail', value)
  }
})

function toggleRail() {
  railModel.value = !railModel.value
}

function expandDrawer() {
  if (railModel.value) {
    railModel.value = false
  }
}
</script>
