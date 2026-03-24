"use client";

import React, { createContext, useContext } from "react";
import useSWR from "swr";
import { Config } from "@/types/config";

const fetcher = (url: string) => fetch(url).then((res) => res.json());

interface PlatformConfigContextValue {
  config?: Config;
  isLoading: boolean;
  error?: any;
}

const PlatformConfigContext = createContext<PlatformConfigContextValue>({
  config: undefined,
  isLoading: true,
  error: undefined,
});

export const PlatformConfigProvider: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const { data, error, isLoading } = useSWR<Config>("/api/config", fetcher, {
    revalidateOnFocus: false,
  });

  return (
    <PlatformConfigContext.Provider value={{ config: data, isLoading, error }}>
      {children}
    </PlatformConfigContext.Provider>
  );
};

export const usePlatformConfig = () => useContext(PlatformConfigContext);

export default PlatformConfigContext;
