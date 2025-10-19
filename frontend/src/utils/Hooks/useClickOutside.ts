import { useEffect } from 'react';
import type { RefObject } from 'react';

function useClickOutside(ref: RefObject<HTMLElement | null>, onClickOutside: () => void): void {
  useEffect(() => {
    /**
     * Invoke Function onClick outside of element
     */
    function handleClickOutside(event: MouseEvent): void {
      if (ref.current && !ref.current.contains(event.target as Node)) {
        onClickOutside();
      }
    }
    // Bind
    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      // dispose
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, [ref, onClickOutside]);
}

export default useClickOutside;
