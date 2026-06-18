import { z } from 'zod'

const VarName = z.string().regex(/^\$[a-zA-Z_][a-zA-Z0-9_]*$/)

export const GivenDirAlbum = z.object({
  dir_album: z.string(),
  id_as: VarName.optional()
}).strict()

export const GivenPhoto = z.object({
  photo: z.string(),
  id_as: VarName.optional(),
  tags: z.array(z.string()).optional(),
  exif_date: z.string().optional(),
  color: z.array(z.number().int().min(0).max(255)).length(3).optional()
}).strict()

export const GivenEmpty = z.object({
  empty: z.literal(true)
}).strict()

export const GivenRemove = z.object({
  remove: z.string()
}).strict()

export const GivenConfig = z.object({
  config: z.object({ read_only_mode: z.boolean() })
}).strict()

export const GivenItem = z.union([
  GivenDirAlbum,
  GivenPhoto,
  GivenEmpty,
  GivenRemove,
  GivenConfig
])

const RoleLabel = z.string()

export const UiWhenNavigate = z.object({
  navigate: z.string()
}).strict()

export const UiWhenClick = z.object({
  click: RoleLabel
}).strict()

export const UiWhenFill = z.object({
  fill: RoleLabel,
  value: z.string()
}).strict()

export const UiWhenSelect = z.object({
  select: RoleLabel,
  option: z.string()
}).strict()

export const UiWhenSubmit = z.object({
  submit: z.literal(true)
}).strict()

export const UiWhenItem = z.union([
  UiWhenNavigate,
  UiWhenClick,
  UiWhenFill,
  UiWhenSelect,
  UiWhenSubmit
])

export const UiThenVisible = z.object({
  'ui.visible': RoleLabel
}).strict()

export const UiThenHidden = z.object({
  'ui.hidden': RoleLabel
}).strict()

export const UiThenText = z.object({
  'ui.text': RoleLabel,
  contains: z.string()
}).strict()

export const UiThenToast = z.object({
  'ui.toast': z.object({
    type: z.enum(['error', 'success', 'warning']),
    contains: z.string()
  })
}).strict()

export const UiThenModal = z.object({
  'ui.modal': z.enum(['open', 'closed'])
}).strict()

export const UiThenRoute = z.object({
  'ui.route': z.string()
}).strict()

export const UiThenAriaSnapshot = z.object({
  'ui.aria_snapshot': z.string()
}).strict()

export const UiThenItem = z.union([
  UiThenVisible,
  UiThenHidden,
  UiThenText,
  UiThenToast,
  UiThenModal,
  UiThenRoute,
  UiThenAriaSnapshot
])

export const UiScenario = z.object({
  name: z.string(),
  given: z.array(GivenItem).optional().default([]),
  when: z.array(UiWhenItem).min(1),
  then: z.array(UiThenItem).min(1)
}).strict()

export type UiScenario = z.infer<typeof UiScenario>
export type UiWhenItem = z.infer<typeof UiWhenItem>
export type UiThenItem = z.infer<typeof UiThenItem>
export type GivenItem = z.infer<typeof GivenItem>
