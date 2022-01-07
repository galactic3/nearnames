import { useContext, useEffect, useState } from "react";
import { ConfirmContext } from "../Providers/ConfirmContextProvider";

const useConfirm = () => {
  const [confirm, setConfirm] = useContext(ConfirmContext);
  const [needsCleanup, setNeedsCleanup] = useState(false);

  const isConfirmed = (prompt) => {
    const promise = new Promise((resolve, reject) => {
      confirm && setConfirm({ prompt, isOpen: true, proceed: resolve, cancel: reject });
      setNeedsCleanup(true);
    });

    const reset = () => {
      confirm && setConfirm({ prompt: "", proceed: null, cancel: null, isOpen: false });
      setNeedsCleanup(false);
    };

    return promise.then(
      () => {
        reset();
        return true;
      },
      () => {
        reset();
        return false;
      }
    );
  };

  //Call cancel in a cleanup func to avoid dangling confirm dialog
  useEffect(() => {
    return () => {
      if (confirm && confirm.cancel && needsCleanup) {
        confirm.cancel();
      }
    };
  }, [confirm, needsCleanup]);

  return {
    ...confirm,
    isConfirmed
  };
};

export default useConfirm;
