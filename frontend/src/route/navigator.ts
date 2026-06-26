import type { Router } from 'vue-router'

export async function navigateToAlbum(albumId: string, router: Router): ReturnType<Router['push']> {
  return router.push({ path: `/album/${albumId}` })
}
