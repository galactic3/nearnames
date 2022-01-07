import React from 'react';
import {Box, IconButton, Modal} from "@mui/material";
import CloseIcon from '@mui/icons-material/Close';
import useConfirm from "../Hooks/useConfirm";

function ModalConfirm (props) {

  const {
    prompt = "",
    isOpen = false,
    proceed,
    cancel
  } = useConfirm();

  return (
    <Modal open={isOpen}>
      <Box className="modal-container alert_modal">
      <IconButton
        aria-label="close"
        onClick={cancel}
        className="button-icon"
        sx={{
          position: 'absolute',
          right: 8,
          top: 8,
          color: 'var(--gray)',
        }}
      >
        <CloseIcon />
      </IconButton>
        <h3>Warning</h3>
        <div className="alert-content">{prompt}</div>
        <div className="button_wrapper">
          <button className="outlined" onClick={cancel}>Cancel</button>
          <button className="" onClick={proceed}>Continue</button>
        </div>
      </Box>
    </Modal>
  )
}

export default ModalConfirm;