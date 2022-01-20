import React from 'react'
import { CircularProgress } from "@mui/material";

export default function Loader (props) {
  return (
    <div className="spinner">
      <CircularProgress sx={{
        color: 'var(--link)',
        margin: 'auto',
        ...props
      }}/>
    </div>
  )
}