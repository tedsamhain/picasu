<template>
  <v-overlay
    v-model="overlayVisible"
    :height="'100%'"
    :width="'100%'"
    class="d-flex"
    id="view-page"
    :transition="false"
    :close-on-back="false"
  >
    <Home
      v-if="album !== undefined && basicString !== null"
      :key="isolatedHomeKey"
      isolation-id="subId"
      :basic-string="basicString"
      :search-string="searchString"
    >
      <template #home-toolbar>
        <HomeIsolatedBar :album="album" />
        <div v-if="childAlbums.length > 0" class="child-albums-strip px-3 py-2 bg-surface">
          <div class="text-caption text-medium-emphasis mb-1">Sub-albums</div>
          <div class="d-flex flex-wrap gap-2">
            <v-chip
              v-for="child in childAlbums"
              :key="child.albumId"
              variant="tonal"
              color="primary"
              size="small"
              style="cursor: pointer"
              @click="navigateToChildAlbum(child.albumId)"
            >
              {{ child.displayName }}
            </v-chip>
          </div>
        </div>
      </template>
    </Home>
  </v-overlay>
</template>
<script setup lang="ts">
import Home from './Home.vue'
import HomeIsolatedBar from '@/components/NavBar/HomeBars/HomeIsolatedBar.vue'
import { GalleryAlbum } from '@type/types'
import { computed, onBeforeMount, Ref, ref, watch } from 'vue'
import { useCollectionStore } from '@/store/collectionStore'
import { LocationQueryValue, useRoute, useRouter } from 'vue-router'
import { useDataStore } from '@/store/dataStore'
import { useAlbumStore } from '@/store/albumStore'

const route = useRoute()
const router = useRouter()
const dataStore = useDataStore('mainId')
const albumStore = useAlbumStore('mainId')
const album: Ref<GalleryAlbum | undefined> = ref(undefined)
const basicString: Ref<string | null> = ref(null)

const searchString = ref<LocationQueryValue | LocationQueryValue[] | undefined>(null)
const collectionStore = useCollectionStore('subId')

const overlayVisible = computed<boolean>({
  get() {
    return true
  },
  set(val: boolean) {
    if (!val) {
      if (collectionStore.editModeOn) {
        collectionStore.editModeOn = false
      } else {
        router.back()
      }
    }
  }
})

const subSearchKey = computed(() => {
  const v = route.query.subSearch
  if (typeof v === 'string') return v
  if (Array.isArray(v)) return v.join(',')
  return ''
})

const locateKey = computed(() => {
  const v = route.query.locate
  if (typeof v === 'string') return v
  if (Array.isArray(v)) return v.join(',')
  return ''
})

const priorityKey = computed(() => {
  const v = route.query.priority_id
  if (typeof v === 'string') return v
  if (Array.isArray(v)) return v.join(',')
  return ''
})

const reverseKey = computed(() => {
  const v = route.query.reverse
  if (typeof v === 'string') return v
  if (Array.isArray(v)) return v.join(',')
  return ''
})

const hashKey = computed(() => (typeof route.params.hash === 'string' ? route.params.hash : ''))

// Child dir-albums whose parentAlbumId matches the current album.
const childAlbums = computed(() => {
  if (!album.value || !albumStore.fetched) return []
  const currentId = album.value.id
  return [...albumStore.albums.values()].filter((a) => a.parentAlbumId === currentId)
})

function navigateToChildAlbum(childId: string) {
  void router.push({ name: 'albumsReadPage', params: { hash: childId } })
}

// This forces ONLY the isolated Home to remount when subSearch changes
const isolatedHomeKey = computed(() => {
  return `isolated-${hashKey.value}-${subSearchKey.value}-${locateKey.value}-${priorityKey.value}-${reverseKey.value}`
})

watch(
  () => route.query.subSearch,
  (v) => {
    searchString.value = v
  },
  { immediate: true }
)

onBeforeMount(() => {
  const hash = route.params.hash
  if (typeof hash === 'string') {
    const index = dataStore.hashMapData.get(hash)
    if (index !== undefined) {
      const data = dataStore.data.get(index)
      if (data?.type === 'album') {
        album.value = data
      }
    }
  }

  const album_id = route.params.hash
  if (typeof album_id === 'string') {
    basicString.value = `and(album:"${album_id}", trashed:false)`
  }

  if (!albumStore.fetched) {
    void albumStore.fetchAlbums()
  }
})
</script>

<style scoped>
.child-albums-strip {
  border-bottom: 1px solid rgba(var(--v-border-color), var(--v-border-opacity));
}
</style>
