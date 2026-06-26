<template>
  <v-col cols="12">
    <v-card border flat>
      <v-card-title class="font-weight-bold">Frontend</v-card-title>
      <v-divider thickness="4" variant="double"></v-divider>

      <v-list-item class="pt-4">
        <v-list-item-title class="mb-2">Thumbnail size</v-list-item-title>
        <v-slider
          show-ticks="always"
          v-model="subRowHeightScaleValue"
          :min="250"
          :max="450"
          :step="10"
          hide-details
          thumb-size="16"
          prepend-icon="mdi-minus"
          append-icon="mdi-plus"
          color="primary"
          @click:prepend="onSubRowHeightScaleUpdate(-10)"
          @click:append="onSubRowHeightScaleUpdate(10)"
        ></v-slider>
      </v-list-item>

      <v-divider></v-divider>

      <v-list-item
        title="Show Filename Chip"
        @click="onShowFilenameChipUpdate(!showFilenameChipValue)"
      >
        <template #append>
          <v-switch
            :model-value="showFilenameChipValue"
            @update:model-value="onShowFilenameChipUpdate"
            color="primary"
            inset
            hide-details
            @click.stop
          ></v-switch>
        </template>
      </v-list-item>

      <v-divider></v-divider>

      <v-list-item @click="onViewBarOverlayUpdate(!viewBarOverlayValue)">
        <template #title>
          <div class="d-flex align-center">
            ViewBar Overlay
            <v-tooltip location="top" max-width="300">
              <template #activator="{ props }">
                <v-icon
                  v-bind="props"
                  icon="mdi-alert-circle-outline"
                  size="small"
                  color="medium-emphasis"
                  class="ml-2"
                  style="cursor: help"
                ></v-icon>
              </template>
              <span>
                <b>When viewing photos:</b><br />
                <b>On:</b> The navigation bar overlays the image.<br />
                <b>Off:</b> The image is pushed down to prevent obstruction.
              </span>
            </v-tooltip>
          </div>
        </template>
        <template #append>
          <v-switch
            :model-value="viewBarOverlayValue"
            @update:model-value="onViewBarOverlayUpdate"
            color="primary"
            inset
            hide-details
            @click.stop
          ></v-switch>
        </template>
      </v-list-item>
    </v-card>
  </v-col>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useConstStore } from '@/store/constStore'

const constStore = useConstStore('mainId')

const subRowHeightScaleValue = computed<number>({
  get: () => constStore.subRowHeightScale,
  set: (newVal: number | null) => {
    const value = newVal ?? constStore.subRowHeightScale
    const clamped = Math.max(250, Math.min(450, value))
    constStore.updateSubRowHeightScale(clamped).catch((error: unknown) => {
      console.error('Failed to update subRowHeightScale (via setter):', error)
    })
  }
})

const showFilenameChipValue = computed<boolean>({
  get: () => constStore.showFilenameChip,
  set: (newVal: boolean | null) => {
    constStore.updateShowFilenameChip(newVal ?? true).catch((error: unknown) => {
      console.error('Failed to update showFilenameChip (via setter):', error)
    })
  }
})

const onSubRowHeightScaleUpdate = (newValue: number | null) => {
  const value = newValue ?? constStore.subRowHeightScale
  const clamped = Math.max(250, Math.min(450, value))
  constStore.updateSubRowHeightScale(clamped).catch((error: unknown) => {
    console.error('Failed to update subRowHeightScale:', error)
  })
}

const onShowFilenameChipUpdate = (newValue: boolean | null) => {
  constStore.updateShowFilenameChip(newValue ?? true).catch((error: unknown) => {
    console.error('Failed to update showFilenameChip:', error)
  })
}

const viewBarOverlayValue = computed<boolean>({
  get: () => constStore.viewBarOverlay,
  set: (newVal: boolean | null) => {
    if (newVal !== null) {
      constStore.updateViewBarOverlay(newVal).catch((error: unknown) => {
        console.error('Failed to update viewBarOverlay (via setter):', error)
      })
    }
  }
})

const onViewBarOverlayUpdate = (newValue: boolean | null) => {
  if (newValue !== null) {
    constStore.updateViewBarOverlay(newValue).catch((error: unknown) => {
      console.error('Failed to update viewBarOverlay:', error)
    })
  }
}
</script>
