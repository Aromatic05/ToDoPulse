<template>
  <v-card class="pa-4">
    <h3 class="text-h6 mb-5">模型设置</h3>
    <v-switch 
      v-model="localAigcEnabled" 
      label="智能生成标签" 
      color="primary" 
      hide-details
      class="mb-4"
      @update:model-value="updateSettings"
    ></v-switch>
    <v-expand-transition>
      <div v-if="localAigcEnabled">
        <v-text-field
          v-model="localToken"
          label="API Token"
          variant="outlined"
          density="compact"
          class="mt-2"
          hide-details
          @update:model-value="updateSettings"
        ></v-text-field>
        <v-text-field
          v-model="localModel"
          label="模型名称 (如 gpt-3.5-turbo)"
          variant="outlined"
          density="compact"
          class="mt-2"
          hide-details
          @update:model-value="updateSettings"
        ></v-text-field>
      </div>
    </v-expand-transition>
  </v-card>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';

const props = defineProps({
  aigcEnabled: {
    type: Boolean,
    default: false
  },
  token: {
    type: String,
    default: ''
  },
  model: {
    type: String,
    default: ''
  }
});

const emit = defineEmits([
  'update:aigcEnabled',
  'update:token',
  'update:model'
]);

const localAigcEnabled = ref(props.aigcEnabled);
const localToken = ref(props.token);
const localModel = ref(props.model);

watch(() => props.aigcEnabled, (val) => {
  localAigcEnabled.value = val;
});
watch(() => props.token, (val) => {
  localToken.value = val;
});
watch(() => props.model, (val) => {
  localModel.value = val;
});

const updateSettings = () => {
  emit('update:aigcEnabled', localAigcEnabled.value);
  emit('update:token', localToken.value);
  emit('update:model', localModel.value);
};
</script>