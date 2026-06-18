import * as fs from 'fs'
import * as path from 'path'
import { GivenItem } from './types'
import type { APIRequestContext } from '@playwright/test'
import { IMAGE_HOME, BACKEND_URL, ADMIN_PASSWORD } from './paths'

let authToken: string | null = null

async function ensureAuthenticated(request: APIRequestContext): Promise<string> {
  if (authToken) return authToken
  const res = await request.post(`${BACKEND_URL}/post/authenticate`, {
    data: ADMIN_PASSWORD
  })
  if (!res.ok()) throw new Error(`Auth failed: ${res.status()}`)
  authToken = String(await res.json())
  return authToken
}

const MINIMAL_JPEG = Buffer.from([
  0xff, 0xd8, 0xff, 0xe0, 0x00, 0x10, 0x4a, 0x46, 0x49, 0x46, 0x00, 0x01,
  0x01, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0xff, 0xdb, 0x00, 0x43,
  0x00, 0x08, 0x06, 0x06, 0x07, 0x06, 0x05, 0x08, 0x07, 0x07, 0x07, 0x09,
  0x09, 0x08, 0x0a, 0x0c, 0x14, 0x0d, 0x0c, 0x0b, 0x0b, 0x0c, 0x19, 0x12,
  0x13, 0x0f, 0x14, 0x1d, 0x1a, 0x1f, 0x1e, 0x1d, 0x1a, 0x1c, 0x1c, 0x20,
  0x24, 0x2e, 0x27, 0x20, 0x22, 0x2c, 0x23, 0x1c, 0x1c, 0x28, 0x37, 0x29,
  0x2c, 0x30, 0x31, 0x34, 0x34, 0x34, 0x1f, 0x27, 0x39, 0x3d, 0x38, 0x32,
  0x3c, 0x2e, 0x33, 0x34, 0x32, 0xff, 0xc0, 0x00, 0x0b, 0x08, 0x00, 0x01,
  0x00, 0x01, 0x01, 0x01, 0x11, 0x00, 0xff, 0xc4, 0x00, 0x1f, 0x00, 0x00,
  0x01, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09,
  0x0a, 0x0b, 0xff, 0xc4, 0x00, 0xb5, 0x10, 0x00, 0x02, 0x01, 0x03, 0x03,
  0x02, 0x04, 0x03, 0x05, 0x05, 0x04, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00,
  0x01, 0x02, 0x03, 0x11, 0x04, 0x05, 0x21, 0x31, 0x06, 0x12, 0x41, 0x51,
  0x07, 0x61, 0x71, 0x13, 0x22, 0x32, 0x81, 0x08, 0x14, 0x42, 0x91, 0xa1,
  0xb1, 0xc1, 0x09, 0x23, 0x33, 0x52, 0xf0, 0x15, 0x62, 0x72, 0xd1, 0x0a,
  0x16, 0xe1, 0xff, 0xd9
])

export interface GivenContext {
  vars: Record<string, string>
}

export function createGivenContext(): GivenContext {
  return { vars: {} }
}

export async function executeGiven(
  request: APIRequestContext,
  given: GivenItem[],
  ctx: GivenContext
): Promise<GivenContext> {
  const result: GivenContext = { vars: { ...ctx.vars } }
  const createdFiles: string[] = []

  for (const item of given) {
    if ('dir_album' in item && item.dir_album) {
      const ga = item as { dir_album: string; id_as?: string }
      const dirPath = path.join(IMAGE_HOME, ga.dir_album)
      fs.mkdirSync(dirPath, { recursive: true })
      if (ga.id_as) {
        const ph = path.join(dirPath, '.__e2e_ph__.jpg')
        fs.writeFileSync(ph, MINIMAL_JPEG)
        createdFiles.push(ga.dir_album)
      }
    }

    if ('photo' in item && item.photo) {
      const ph = item as { photo: string; id_as?: string }
      const filePath = path.join(IMAGE_HOME, ph.photo)
      fs.mkdirSync(path.dirname(filePath), { recursive: true })
      fs.writeFileSync(filePath, MINIMAL_JPEG)
      createdFiles.push(ph.photo)
    }

    if ('remove' in item && item.remove) {
      const filePath = path.join(IMAGE_HOME, item.remove)
      try { fs.unlinkSync(filePath) } catch { }
    }
  }

  if (createdFiles.length > 0) {
    const token = await ensureAuthenticated(request)
    const authHeaders = { Authorization: `Bearer ${token}` }

    const indexRes = await request.fetch(`${BACKEND_URL}/post/index/album`, {
      method: 'POST',
      headers: { ...authHeaders, 'Content-Type': 'application/json' },
      data: { album: '/' }
    })
    if (!indexRes.ok()) {
      throw new Error(`Index failed: ${indexRes.status()} ${await indexRes.text()}`)
    }

    let indexed = false
    for (let i = 0; i < 60; i++) {
      const statusRes = await request.fetch(`${BACKEND_URL}/get/index/status`, {
        headers: authHeaders
      })
      if (statusRes.ok()) {
        const status = await statusRes.json()
        if (status.indexing === false) { indexed = true; break }
      }
      await new Promise(r => setTimeout(r, 1000))
    }
    if (!indexed) throw new Error('Index did not complete within 60s')

    if (given.some(g => 'id_as' in (g as any) && (g as any).id_as)) {
      const dataRes = await request.fetch(
        `${BACKEND_URL}/get/get-data?start=0&end=100`,
        { headers: authHeaders }
      )
      const data = await dataRes.json() as any[]

      for (const item of given) {
        if ('dir_album' in item && item.dir_album && (item as any).id_as) {
          const ga = item as { dir_album: string; id_as: string }
          const album = await findAlbum(request, authHeaders, ga.dir_album)
          if (album) result.vars[ga.id_as] = album
        }
        if ('photo' in item && item.photo && (item as any).id_as) {
          const ph = item as { photo: string; id_as: string }
          const hash = await findPhotoHash(ph.photo, data)
          if (hash) result.vars[ph.id_as] = hash
        }
      }
    }
  }

  return result
}

async function findAlbum(
  request: APIRequestContext,
  headers: Record<string, string>,
  dirPath: string
): Promise<string | null> {
  const res = await request.fetch(`${BACKEND_URL}/get/get-albums`, { headers })
  if (!res.ok()) return null
  const albums = await res.json() as any[]
  const match = albums.find((a: any) => a.dirPath && a.dirPath.endsWith(dirPath))
  return match ? String(match.album_id) : null
}

function findPhotoHash(photoPath: string, data: any[]): string | null {
  const match = data.find((d: any) => {
    const alias = d.abstractData?.currentAlias?.filePath
    return alias && alias.endsWith(photoPath)
  })
  return match ? String(match.hash) : null
}
