import { getScrollUpperBound } from '@utils/getter'
import { IsolationId } from '@type/types'
import { usePrefetchStore } from '@/store/prefetchStore'
import { useScrollTopStore } from '@/store/scrollTopStore'
import { Ref } from 'vue'
import { useConfigStore } from '@/store/configStore'

/**
 * Immediate scroll handler that compensates native scroll changes and updates the virtual scroll position.
 * This function compensates for changes in `imageContainerRef.value.scrollTop` caused by user scrolling,
 * ensuring the scroll position remains within `bufferHeight.value / 3`, as initialized in `initializeScrollPosition.ts`.
 *
 * @param imageContainerRef - Reference to the scrolling container element.
 * @param lastScrollTop - Reference to the last recorded scroll position.
 * @param scrollTop - Reference to the current scroll position.
 * @param stopScroll - Flag to temporarily stop scrolling for mobile adjustments.
 * @param windowHeight - Reference to the window height for scroll limit calculations.
 *
 * @returns Scroll event handler + wheel boundary guard.
 */
export function handleScroll(
  imageContainerRef: Ref<HTMLElement | null>,
  lastScrollTop: Ref<number>,
  stopScroll: Ref<boolean>,
  windowHeight: Ref<number>,
  isolationId: IsolationId
) {
  function throttledHandleScroll() {
    if (imageContainerRef.value !== null) {
      const configStore = useConfigStore('mainId')
      const mobile = configStore.isMobile
      const scrollTopStore = useScrollTopStore(isolationId)
      const prefetchStore = usePrefetchStore(isolationId)

      const difference = imageContainerRef.value.scrollTop - lastScrollTop.value

      if (prefetchStore.totalHeight - windowHeight.value < 0) {
        if (mobile) {
          stopScroll.value = true
          scrollTopStore.scrollTop = 0
          setTimeout(() => {
            stopScroll.value = false
          }, 100)
        } else {
          scrollTopStore.scrollTop = 0
        }
        imageContainerRef.value.scrollTop -= difference
        lastScrollTop.value = imageContainerRef.value.scrollTop
        return
      }

      const result = scrollTopStore.scrollTop + difference

      if (result < 0) {
        if (mobile) {
          stopScroll.value = true
          scrollTopStore.scrollTop = 0
          setTimeout(() => {
            stopScroll.value = false
          }, 100)
        } else {
          scrollTopStore.scrollTop = 0
        }
      } else if (result >= getScrollUpperBound(prefetchStore.totalHeight, windowHeight.value)) {
        if (mobile) {
          stopScroll.value = true
          scrollTopStore.scrollTop = getScrollUpperBound(
            prefetchStore.totalHeight,
            windowHeight.value
          )
          setTimeout(() => {
            stopScroll.value = false
          }, 100)
        } else {
          scrollTopStore.scrollTop = getScrollUpperBound(
            prefetchStore.totalHeight,
            windowHeight.value
          )
        }
      } else {
        scrollTopStore.scrollTop += difference
      }

      imageContainerRef.value.scrollTop -= difference
      lastScrollTop.value = imageContainerRef.value.scrollTop
    }
  }

  /**
   * Wheel event handler that fires before the browser applies native scrolling.
   * Uses `preventDefault()` to block scrolls that would pass the virtual boundary,
   * eliminating the initial movement + snap-back that creates the flicker.
   * Listener must be registered with `{ passive: false }` for `preventDefault()` to work.
   */
  function onWheel(e: WheelEvent) {
    if (imageContainerRef.value === null) return

    const scrollTopStore = useScrollTopStore(isolationId)
    const prefetchStore = usePrefetchStore(isolationId)

    if (prefetchStore.locateTo !== null) return

    if (prefetchStore.totalHeight - windowHeight.value < 0) {
      e.preventDefault()
      return
    }

    let estimatedDelta = e.deltaY
    if (e.deltaMode === 1) {
      estimatedDelta *= 16
    } else if (e.deltaMode === 2) {
      estimatedDelta *= windowHeight.value
    }

    const upperBound = getScrollUpperBound(prefetchStore.totalHeight, windowHeight.value)
    const virtualPos = scrollTopStore.scrollTop

    if (estimatedDelta < 0 && virtualPos <= 1) {
      e.preventDefault()
      return
    }

    if (estimatedDelta > 0 && virtualPos >= upperBound) {
      e.preventDefault()
      return
    }

    if (estimatedDelta < 0 && virtualPos + estimatedDelta < 0) {
      e.preventDefault()
      scrollTopStore.scrollTop = 0
      return
    }

    if (estimatedDelta > 0 && virtualPos + estimatedDelta >= upperBound) {
      e.preventDefault()
      scrollTopStore.scrollTop = upperBound
      return
    }
  }

  return { throttledHandleScroll, onWheel }
}
